// Web Agent Runtime — Embedded LLM + Agent Residency in Notebook
// Enables live interaction with notebook cells and persistent agent state

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent persona (lives in notebook)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPersona {
    /// Agent identifier (e.g., "carto", "resonance", "phantom")
    pub agent_id: String,

    /// Agent name (human-readable)
    pub agent_name: String,

    /// System prompt (defines behavior)
    pub system_prompt: String,

    /// Agent model (e.g., "claude-opus", "gpt-4", "local")
    pub model: String,

    /// Agent context window (max tokens)
    pub context_window: u32,

    /// Current state (JSON-serialized)
    pub state: String,

    /// Memory buffer (recent interactions)
    pub memory_buffer: Vec<String>,

    /// Capabilities (what the agent can do)
    pub capabilities: Vec<String>,

    /// Is agent active (can accept requests)
    pub active: bool,
}

impl AgentPersona {
    pub fn new(
        agent_id: String,
        agent_name: String,
        system_prompt: String,
        model: String,
        context_window: u32,
        capabilities: Vec<String>,
    ) -> Self {
        AgentPersona {
            agent_id,
            agent_name,
            system_prompt,
            model,
            context_window,
            state: "{}".to_string(),
            memory_buffer: Vec::new(),
            capabilities,
            active: true,
        }
    }

    /// Add to memory buffer (circular buffer, max 10 items)
    pub fn remember(&mut self, message: String) {
        self.memory_buffer.push(message);
        if self.memory_buffer.len() > 10 {
            self.memory_buffer.remove(0);
        }
    }

    /// Get agent's recent context (for API calls)
    pub fn get_context(&self) -> String {
        format!(
            "Agent: {}\nModel: {}\nCapabilities: {:?}\nRecent:\n{}",
            self.agent_name,
            self.model,
            self.capabilities,
            self.memory_buffer.join("\n")
        )
    }
}

/// Web chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Who sent it (user, agent_id, or system)
    pub sender: String,

    /// Message content
    pub content: String,

    /// Timestamp (Unix seconds)
    pub timestamp: u64,

    /// Message role (user, assistant, system)
    pub role: String,
}

/// Web agent runtime (lives in notebook, responds to queries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAgentRuntime {
    /// All agent personas
    agents: HashMap<String, AgentPersona>,

    /// Chat history
    chat_history: Vec<ChatMessage>,

    /// Current active agent
    active_agent: Option<String>,

    /// Web session ID
    session_id: String,

    /// Is runtime accepting new chats
    accepting_chat: bool,
}

impl WebAgentRuntime {
    pub fn new(session_id: String) -> Self {
        WebAgentRuntime {
            agents: HashMap::new(),
            chat_history: Vec::new(),
            active_agent: None,
            session_id,
            accepting_chat: true,
        }
    }

    /// Register agent persona in notebook
    pub fn register_agent(&mut self, persona: AgentPersona) -> Result<(), String> {
        if self.agents.contains_key(&persona.agent_id) {
            return Err(format!("Agent {} already registered", persona.agent_id));
        }
        self.agents.insert(persona.agent_id.clone(), persona);
        Ok(())
    }

    /// Set active agent (the one responding to chat)
    pub fn set_active_agent(&mut self, agent_id: &str) -> Result<(), String> {
        if !self.agents.contains_key(agent_id) {
            return Err(format!("Agent {} not found", agent_id));
        }

        let agent = &self.agents[agent_id];
        if !agent.active {
            return Err(format!("Agent {} is not active", agent_id));
        }

        self.active_agent = Some(agent_id.to_string());
        Ok(())
    }

    /// Send message to active agent
    pub fn send_message(&mut self, user_id: String, content: String) -> Result<String, String> {
        if !self.accepting_chat {
            return Err("Runtime is not accepting chat".to_string());
        }

        let agent_id = self.active_agent.as_ref()
            .ok_or("No active agent")?
            .clone();

        // Add user message to history
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let user_msg = ChatMessage {
            sender: user_id.clone(),
            content: content.clone(),
            timestamp: now,
            role: "user".to_string(),
        };

        self.chat_history.push(user_msg);

        // Get agent
        let agent = self.agents.get_mut(&agent_id)
            .ok_or("Agent not found")?;

        // Add to agent memory
        agent.remember(format!("User: {}", content));

        // Simulate agent response (in production, call actual LLM)
        let response = format!(
            "Agent {} received: '{}'. Processing...",
            agent.agent_name, content
        );

        let agent_msg = ChatMessage {
            sender: agent_id.clone(),
            content: response.clone(),
            timestamp: now,
            role: "assistant".to_string(),
        };

        self.chat_history.push(agent_msg);
        agent.remember(format!("Agent: {}", response));

        Ok(response)
    }

    /// Get chat history
    pub fn get_chat_history(&self) -> &[ChatMessage] {
        &self.chat_history
    }

    /// Get all registered agents
    pub fn list_agents(&self) -> Vec<AgentPersona> {
        self.agents.values().cloned().collect()
    }

    /// Get agent by ID
    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentPersona> {
        self.agents.get(agent_id)
    }

    /// Export agents to notebook metadata
    pub fn export_agents(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.agents)
            .map_err(|e| format!("Failed to export agents: {}", e))
    }

    /// Import agents from notebook metadata
    pub fn import_agents(&mut self, json: &str) -> Result<(), String> {
        let agents: HashMap<String, AgentPersona> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to import agents: {}", e))?;
        self.agents = agents;
        Ok(())
    }
}

impl Default for WebAgentRuntime {
    fn default() -> Self {
        use uuid::Uuid;
        Self::new(Uuid::new_v4().to_string())
    }
}

/// Web LLM bridge (connects notebook to external LLM APIs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebLLMBridge {
    /// LLM provider (openai, anthropic, local, etc.)
    pub provider: String,

    /// API endpoint
    pub endpoint: String,

    /// Model name
    pub model: String,

    /// Max retries
    pub max_retries: u32,

    /// Request timeout (seconds)
    pub timeout_secs: u32,
}

impl WebLLMBridge {
    pub fn new(provider: String, endpoint: String, model: String) -> Self {
        WebLLMBridge {
            provider,
            endpoint,
            model,
            max_retries: 3,
            timeout_secs: 30,
        }
    }

    /// Prepare API request (in production, make actual HTTP call)
    pub fn prepare_request(&self, messages: &[ChatMessage]) -> Result<String, String> {
        let payload = serde_json::json!({
            "model": self.model,
            "messages": messages.iter().map(|m| serde_json::json!({
                "role": m.role,
                "content": m.content
            })).collect::<Vec<_>>(),
            "max_tokens": 2048,
        });

        Ok(payload.to_string())
    }

    /// Simulate LLM call (placeholder)
    pub fn call_llm(&self, messages: &[ChatMessage]) -> Result<String, String> {
        let request = self.prepare_request(messages)?;

        // In production, this would make HTTP request to self.endpoint
        // For now, return simulated response
        Ok(format!(
            "LLM Response from {}/{}: Processing {} messages...",
            self.provider,
            self.model,
            messages.len()
        ))
    }
}

/// HTML/JS widget for embedding in README
pub fn generate_web_agent_widget() -> String {
    r#"
<!-- SOVEREIGN NOTEBOOK WEB AGENT WIDGET -->
<div id="sovereign-notebook-chat" style="font-family: monospace; max-width: 600px; border: 2px solid #00ff00; padding: 10px; background: #0a0a0a; color: #00ff00;">
  <div style="margin-bottom: 10px; font-weight: bold;">
    🤖 SOVEREIGN NOTEBOOK AGENTS
  </div>

  <div id="agent-selector" style="margin-bottom: 10px;">
    <label>Select Agent:</label>
    <select id="agent-select" style="margin-left: 5px;">
      <option value="carto">CARTO (Cartographer)</option>
      <option value="resonance">RESONANCE (Math Engine)</option>
      <option value="phantom">PHANTOM (Formal Prover)</option>
    </select>
  </div>

  <div id="chat-history" style="height: 300px; overflow-y: auto; border: 1px solid #00ff00; margin-bottom: 10px; padding: 5px; background: #000000;">
    <div style="color: #888;">[Chat history appears here]</div>
  </div>

  <div style="display: flex; gap: 5px;">
    <input id="user-input" type="text" placeholder="Ask the agent..."
           style="flex: 1; padding: 5px; background: #1a1a1a; color: #00ff00; border: 1px solid #00ff00;">
    <button id="send-btn" style="padding: 5px 15px; background: #00ff00; color: #000; cursor: pointer; border: none;">
      SEND
    </button>
  </div>
</div>

<script>
const ChatWidget = {
  activeAgent: 'resonance',

  async sendMessage() {
    const input = document.getElementById('user-input');
    const content = input.value.trim();
    if (!content) return;

    const history = document.getElementById('chat-history');
    history.innerHTML += `<div style="color: #00ff00;">You: ${content}</div>`;

    try {
      const response = await fetch('/api/agents/message', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          agent_id: this.activeAgent,
          content: content
        })
      });

      const data = await response.json();
      history.innerHTML += `<div style="color: #ffff00;">${data.agent}: ${data.response}</div>`;
    } catch (e) {
      history.innerHTML += `<div style="color: #ff0000;">Error: ${e.message}</div>`;
    }

    input.value = '';
    history.scrollTop = history.scrollHeight;
  }
};

document.getElementById('send-btn').addEventListener('click', () => ChatWidget.sendMessage());
document.getElementById('agent-select').addEventListener('change', (e) => {
  ChatWidget.activeAgent = e.target.value;
});
</script>
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_persona_creation() {
        let agent = AgentPersona::new(
            "test-agent".to_string(),
            "Test Agent".to_string(),
            "You are helpful".to_string(),
            "claude-opus".to_string(),
            8192,
            vec!["answer_questions".to_string()],
        );

        assert_eq!(agent.agent_id, "test-agent");
        assert_eq!(agent.model, "claude-opus");
        assert!(agent.active);
    }

    #[test]
    fn test_web_agent_runtime_registration() {
        let mut runtime = WebAgentRuntime::new("session-1".to_string());

        let agent = AgentPersona::new(
            "test".to_string(),
            "Test".to_string(),
            "Prompt".to_string(),
            "claude".to_string(),
            8192,
            vec![],
        );

        assert!(runtime.register_agent(agent.clone()).is_ok());
        assert!(runtime.register_agent(agent).is_err()); // Duplicate
    }

    #[test]
    fn test_web_agent_runtime_chat() {
        let mut runtime = WebAgentRuntime::new("session-1".to_string());

        let agent = AgentPersona::new(
            "test".to_string(),
            "Test Agent".to_string(),
            "Prompt".to_string(),
            "claude".to_string(),
            8192,
            vec![],
        );

        runtime.register_agent(agent).ok();
        runtime.set_active_agent("test").ok();

        let response = runtime.send_message("user1".to_string(), "Hello".to_string());
        assert!(response.is_ok());

        let history = runtime.get_chat_history();
        assert_eq!(history.len(), 2); // User + Agent
    }

    #[test]
    fn test_web_llm_bridge_creation() {
        let bridge = WebLLMBridge::new(
            "anthropic".to_string(),
            "https://api.anthropic.com".to_string(),
            "claude-opus".to_string(),
        );

        assert_eq!(bridge.provider, "anthropic");
        assert_eq!(bridge.model, "claude-opus");
    }

    #[test]
    fn test_web_agent_widget_generation() {
        let widget = generate_web_agent_widget();
        assert!(widget.contains("SOVEREIGN NOTEBOOK AGENTS"));
        assert!(widget.contains("CARTO"));
        assert!(widget.contains("RESONANCE"));
        assert!(widget.contains("PHANTOM"));
    }
}
