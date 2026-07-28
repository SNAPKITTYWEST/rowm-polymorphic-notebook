/**
 * Ahmad Orbital Agent
 * Orchestrates ISS telemetry verification against formal proofs
 * Integrates: BOB VOYAGER (live ISS) + sov-kernel-monster (proofs) + ROWM notebook (UI)
 */

class AhmadOrbitalAgent {
  constructor(options = {}) {
    this.voyagerUrl = options.voyagerUrl || 'http://localhost:4299';
    this.ahmadUrl = options.ahmadUrl || 'http://localhost:5555'; // Ahmad Orchestrator in sov-kernel-monster
    this.notebookBox = document.getElementById('ahmad-jit-box');
    this.verificationLog = [];
    this.state = 'IDLE';
  }

  /**
   * Query Ahmad Orchestrator for verification
   */
  async queryAhmadOrchestrator() {
    try {
      const res = await fetch(`${this.ahmadUrl}/verify`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          timestamp: new Date().toISOString(),
        }),
      });
      return await res.json();
    } catch (e) {
      console.error('[AHMAD-ORBITAL] Orchestrator query failed:', e.message);
      return { ok: false, error: e.message };
    }
  }

  /**
   * Query Ahmad status
   */
  async getAhmadStatus() {
    try {
      const res = await fetch(`${this.ahmadUrl}/status`);
      return await res.json();
    } catch (e) {
      console.error('[AHMAD-ORBITAL] Status query failed:', e.message);
      return { ok: false, error: e.message };
    }
  }

  /**
   * Render verification result in notebook
   */
  renderVerificationResult(result) {
    if (!this.notebookBox) return;

    const box = this.notebookBox.querySelector('.jit-box-content');
    if (!box) return;

    const status = result.valid ? '✓ VERIFIED' : '✗ ANOMALY';
    const statusColor = result.valid ? '#4ade80' : '#f87171';

    const html = `
      <div style="padding: 12px; border-left: 4px solid ${statusColor}; background: rgba(0,0,0,0.05);">
        <div style="font-weight: bold; color: ${statusColor};">${status}</div>
        <div style="font-size: 12px; margin-top: 8px; font-family: monospace; line-height: 1.4;">
          <div>LAT ${result.position[0].toFixed(4)}° | LON ${result.position[1].toFixed(4)}°</div>
          <div>ALT ${result.altitude.toFixed(0)}km | VEL ${result.velocity.toFixed(2)}km/s</div>
          <div>Invariants: ${result.invariants_passed}/${result.invariants_total}</div>
          ${result.errors.length > 0 ? `<div style="color: #f87171; margin-top: 4px;">${result.errors[0]}</div>` : ''}
          <div style="color: #999; margin-top: 4px;">WORM: ${result.seal.hash}</div>
        </div>
      </div>
    `;

    // Append to existing content
    const entry = document.createElement('div');
    entry.innerHTML = html;
    box.appendChild(entry);

    // Keep max 5 recent verifications visible
    const entries = box.querySelectorAll('[style*="border-left"]');
    if (entries.length > 5) {
      entries[0].remove();
    }
  }

  /**
   * Run single verification cycle (via Ahmad Orchestrator)
   */
  async runVerification() {
    this.state = 'VERIFYING';

    const result = await this.queryAhmadOrchestrator();
    if (!result.ok) {
      console.error('[AHMAD-ORBITAL] Ahmad Orchestrator error:', result.error);
      this.state = 'ERROR';
      return { ok: false, error: result.error };
    }

    // Extract verification from orchestrator response
    const verification = result.verification;
    if (!verification) {
      this.state = 'ERROR';
      return { ok: false, error: 'No verification in response' };
    }

    this.verificationLog.push({
      timestamp: new Date().toISOString(),
      decision: result.decision,
      ...verification,
    });

    this.renderVerificationResult(verification);
    this.state = 'READY';

    return verification;
  }

  /**
   * Start continuous orbital verification loop
   * Polls every 5 seconds (aligned with ISS telemetry update rate)
   */
  startOrbitalMonitoring() {
    console.log('[AHMAD-ORBITAL] Starting orbital verification loop...');
    this.state = 'MONITORING';

    this.monitoringInterval = setInterval(async () => {
      const result = await this.runVerification();
      if (result.ok) {
        const status = result.valid ? '✓' : '✗';
        console.log(`[AHMAD-ORBITAL] ${status} ISS at LAT ${result.position[0].toFixed(2)}° ALT ${result.altitude.toFixed(0)}km`);
      }
    }, 5000);
  }

  stopOrbitalMonitoring() {
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.state = 'IDLE';
      console.log('[AHMAD-ORBITAL] Orbital monitoring stopped');
    }
  }

  /**
   * Get verification log (last N entries)
   */
  getLog(limit = 100) {
    return this.verificationLog.slice(-limit);
  }

  /**
   * Export verification history as JSON
   */
  exportLog() {
    return {
      agent: 'Ahmad Orbital Agent',
      timestamp: new Date().toISOString(),
      verifications: this.verificationLog,
      status: this.state,
    };
  }
}

// Global instance
window.ahmadOrbitalAgent = null;

// Export
if (typeof module !== 'undefined' && module.exports) {
  module.exports = AhmadOrbitalAgent;
}
