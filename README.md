<img width="1408" height="768" alt="Carb" src="https://github.com/user-attachments/assets/d2c91cbe-72b1-4942-8e54-eee82334ad4b" />

# Carb
🦀Carb – Open‑source Windows malware research tool. Demonstrates syscall hooking (`NtQuerySystemInformation`) for process hiding, thread explosion (10M threads), Argon2id hashing (100GB memory), GPU compute abuse, and peripheral disabling (mouse/audio/camera). Auto‑persists via Startup folder – never deploy without explicit consent.

## ⚙️ Features – Attack Vectors & Techniques

| **Feature** | **Implementation Technique** | **Impact** | **Research Purpose** |
| :--- | :--- | :--- | :--- |
| 🕵️ **Process Hiding** | Hooks `NtQuerySystemInformation` (syscall) – removes current PID from system process list | Invisible in Task Manager, Process Explorer, and most monitoring tools | Understand how malware evades user‑mode detection |
| ♻️ **Persistence** | Copies binary to `%APPDATA%\...\Startup` folder | Executes automatically on every Windows boot | Study common auto‑start persistence mechanisms |
| 💥 **CPU & RAM Exhaustion** | Spawns **10 million threads** (1MB stack each) + Argon2id hashing (`mem_cost=100GB`, `time=50`, `lanes=3667`) | Saturates all CPU cores; triggers massive pagefile thrashing; system becomes unresponsive | Analyze fork‑bomb logic and memory‑hardened hash abuse |
| 🎮 **GPU Stress** | Infinite loop dispatching compute shaders (WGSL) via `wgpu` | GPU utilization hits 100%; temperature spikes; graphical performance drops to zero | Measure GPU response under sustained compute load |
| 🖥️ **Peripheral Disable** | PowerShell: `Disable-PnpDevice` targeting mouse, audio, and camera drivers | Complete loss of input/control; user cannot interact with the system | Explore driver‑level disabling techniques |
| 🌑 **Screen Blackout** | `SetMonitorBrightness(0)` via Windows Monitor API | Screen goes completely dark (backlight remains on), causing visual denial‑of‑service | Examine low‑level monitor communication |
| ⌨️ **Keyboard Layout Hijack** | `LoadKeyboardLayoutW` + `ActivateKeyboardLayout` to force US English | Confuses user input; makes typing passwords or recovery commands difficult | Study input method manipulation |
| 🔪 **Process Killer** | PowerShell: `Get-Process \| Where-Object {$_.Id -ne $pid} \| Stop-Process -Force` | Terminates all other applications (including Explorer, Defender, Task Manager) | Analyze system takeover and defense disruption tactics |

---

> **📌 DISCLAIMER:** All features above are intentionally designed to **simulate real‑world malware behavior** for controlled testing environments. **Strictly FORBIDDEN** to use on production systems, unauthorized devices, or without explicit consent. Use only in isolated VMs or sandboxed labs. The author assumes **zero liability** for any misuse or damage.

 🦀
<img width="576" height="462" alt="burningyourmom" src="https://github.com/user-attachments/assets/60a548ae-7831-4db3-acae-5ed00a88d8a3" />
<!-- 🔥 CARB – RESOURCE OVERLOAD DASHBOARD (Full Width) -->
<div align="center">
  <h2>🔥 Resource Overload – Peak (t = 2.5s)</h2>
  <table style="width:100%; max-width:900px; border-collapse:collapse; font-size:1.3em; background-color:#0d1117; color:#c9d1d9; border-radius:12px; overflow:hidden; box-shadow:0 8px 24px rgba(0,0,0,0.8);">
    <thead>
      <tr style="background-color:#161b22; border-bottom:2px solid #30363d;">
        <th style="padding:16px 12px; text-align:left; font-size:1.1em; color:#f0f6fc;">Resource</th>
        <th style="padding:16px 12px; text-align:center; font-size:1.1em; color:#f0f6fc;">Usage</th>
        <th style="padding:16px 12px; text-align:center; font-size:1.1em; color:#f0f6fc;">Status</th>
        <th style="padding:16px 12px; text-align:left; font-size:1.1em; color:#f0f6fc;">Bar</th>
      </tr>
    </thead>
    <tbody>
      <tr style="border-bottom:1px solid #21262d;">
        <td style="padding:18px 12px; font-weight:bold; font-size:1.2em;">🚀 CPU</td>
        <td style="padding:18px 12px; text-align:center; font-weight:bold; font-size:1.3em; color:#ff7b72;">100%</td>
        <td style="padding:18px 12px; text-align:center; font-weight:bold; font-size:1.1em; color:#ff7b72;">🔴 MAX</td>
        <td style="padding:18px 12px;"><span style="display:block; background:#ff7b72; width:100%; height:28px; border-radius:6px;"></span></td>
      </tr>
      <tr style="border-bottom:1px solid #21262d; background-color:#161b22;">
        <td style="padding:18px 12px; font-weight:bold; font-size:1.2em;">🧠 RAM (Physical)</td>
        <td style="padding:18px 12px; text-align:center; font-weight:bold; font-size:1.3em; color:#ff7b72;">15.8 / 16 GB</td>
        <td style="padding:18px 12px; text-align:center; font-weight:bold; font-size:1.1em; color:#ff7b72;">🔴 FULL</td>
        <td style="padding:18px 12px;"><span style="display:block; background:#ff7b72; width:99%; height:28px; border-radius:6px;"></span></td>
      </tr>
      <tr style="border-bottom:1px solid #21262d;">
        <td style="padding:18px 12px; font-weight:bold; font-size:1.2em;">💾 RAM (Pagefile)</td>
        <td style="padding:18px 12px; text-align:center; font-weight:bold; font-size:1.3em; color:#ff7b72;">72.0 / 80 GB</td>
        <td style="padding:18px 12px; text-align:center; font-weight:bold; font-size:1.1em; color:#ff7b72;">🔴 CRIT</td>
        <td style="padding:18px 12px;"><span style="display:block; background:#ff7b72; width:90%; height:28px; border-radius:6px;"></span></td>
      </tr>
      <tr style="border-bottom:1px solid #21262d; background-color:#161b22;">
        <td style="padding:18px 12px; font-weight:bold; font-size:1.2em;">🎮 GPU</td>
        <td style="padding:18px 12px; text-align:center; font-weight:bold; font-size:1.3em; color:#f0883e;">98%</td>
        <td style="padding:18px 12px; text-align:center; font-weight:bold; font-size:1.1em; color:#f0883e;">🟡 NEAR</td>
        <td style="padding:18px 12px;"><span style="display:block; background:#f0883e; width:98%; height:28px; border-radius:6px;"></span></td>
      </tr>
      <tr>
        <td style="padding:18px 12px; font-weight:bold; font-size:1.2em;">💿 Disk I/O</td>
        <td style="padding:18px 12px; text-align:center; font-weight:bold; font-size:1.3em; color:#ff7b72;">100%</td>
        <td style="padding:18px 12px; text-align:center; font-weight:bold; font-size:1.1em; color:#ff7b72;">🔴 BUSY</td>
        <td style="padding:18px 12px;"><span style="display:block; background:#ff7b72; width:100%; height:28px; border-radius:6px;"></span></td>
      </tr>
    </tbody>
  </table>
  <p style="font-size:0.95em; color:#8b949e; margin-top:12px;"><em>⚡ Simulated dashboard – hardware results may vary.</em></p>
</div>
<!-- 🔐 Carb – File Hashes -->
<div align="center">
  <h2>🔐 File Hashes – Carb.exe</h2>
  <table style="width:100%; max-width:900px; border-collapse:collapse; font-size:1.1em; background-color:#0d1117; color:#c9d1d9; border-radius:12px; overflow:hidden; box-shadow:0 8px 24px rgba(0,0,0,0.8);">
    <thead>
      <tr style="background-color:#161b22; border-bottom:2px solid #30363d;">
        <th style="padding:16px 12px; text-align:left; font-size:1.1em; color:#f0f6fc; width:30%;">Algorithm</th>
        <th style="padding:16px 12px; text-align:left; font-size:1.1em; color:#f0f6fc;">Hash (Lowercase)</th>
      </tr>
    </thead>
    <tbody>
      <tr style="border-bottom:1px solid #21262d;">
        <td style="padding:18px 12px; font-weight:bold; font-size:1.1em;">🖥️ MD5</td>
        <td style="padding:18px 12px; font-family: 'Courier New', monospace; word-break:break-all; font-size:0.95em;">d3eeec515c96cc5229fc4a1dc53eef7a</td>
      </tr>
      <tr style="border-bottom:1px solid #21262d; background-color:#161b22;">
        <td style="padding:18px 12px; font-weight:bold; font-size:1.1em;">🔑 SHA‑1</td>
        <td style="padding:18px 12px; font-family: 'Courier New', monospace; word-break:break-all; font-size:0.95em;">48b1bf84b31072f21a7ca57943911df11fd65e7a</td>
      </tr>
      <tr>
        <td style="padding:18px 12px; font-weight:bold; font-size:1.1em;">🛡️ SHA‑256</td>
        <td style="padding:18px 12px; font-family: 'Courier New', monospace; word-break:break-all; font-size:0.95em;">26c4cfb4b689af68b71ff273c9eb11e15d6959f92317e1a19fdb84be08f2a41c</td>
      </tr>
    </tbody>
  </table>
  <p style="font-size:0.95em; color:#8b949e; margin-top:12px;">
    💡 Verify with <code style="background:#161b22; padding:4px 10px; border-radius:6px; font-size:0.9em;">CertUtil -hashfile Carb.exe SHA256</code> (Windows) or <code style="background:#161b22; padding:4px 10px; border-radius:6px; font-size:0.9em;">sha256sum Carb.exe</code> (Linux)
  </p>
</div>
# Target

<img width="678" height="452" alt="target" src="https://github.com/user-attachments/assets/040d39ec-cc2b-4dd4-a45c-6bc298e9de32" />
