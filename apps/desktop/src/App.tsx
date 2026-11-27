import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Device = {
  device_path: string;
  stable_id: string | null;
  serial: string | null;
  model: string;
  capacity_bytes: number;
};

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "Capacidade indisponível";
  const units = ["B", "KB", "GB", "TB", "PB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit += 1;
  }
  return `${value.toFixed(value >= 100 ? 0 : 1)} ${units[unit]}`;
}

export default function App() {
  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [scanned, setScanned] = useState(false);

  async function scanDevices() {
    setLoading(true);
    setError("");
    try {
      const found = await invoke<Device[]>("list_storage_devices");
      setDevices(found);
      setScanned(true);
    } catch (cause) {
      setError(String(cause));
      setDevices([]);
      setScanned(true);
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">D</div>
          <div><strong>DECODE</strong><span>DISK DOCTOR</span></div>
        </div>
        <div className="nav-label">WORKSPACE</div>
        <div className="nav-item active"><span>◈</span> Visão geral</div>
        <div className="nav-item muted"><span>▤</span> Diagnóstico</div>
        <div className="nav-item muted"><span>◉</span> SMART / NVMe</div>
        <div className="nav-item muted"><span>▧</span> Backup e imagem</div>
        <div className="nav-item muted"><span>⤴</span> Clonagem</div>
        <div className="nav-item muted"><span>⌕</span> Recuperação</div>
        <div className="sidebar-bottom">
          <span className="status-dot" /> Modo local
          <small>Decode Analytics · v0.1.0</small>
        </div>
      </aside>

      <section className="content">
        <header className="topbar">
          <div><span className="eyebrow">DECODE ANALYTICS / STORAGE</span><h1>Visão geral</h1></div>
          <div className="top-actions"><span className="offline"><i /> OFFLINE READY</span><div className="avatar">DA</div></div>
        </header>

        <section className="hero">
          <div className="hero-copy">
            <span className="eyebrow blue">DISK INTELLIGENCE PLATFORM</span>
            <h2>Seus discos.<br /><em>Sob controle.</em></h2>
            <p>Diagnóstico técnico, integridade e recuperação de dados em um só lugar. Comece identificando os dispositivos conectados.</p>
            <button className="primary" onClick={scanDevices} disabled={loading}>
              {loading ? <><span className="spinner" /> Analisando dispositivos...</> : "⌕  Detectar discos"}
            </button>
          </div>
          <div className="hero-art" aria-hidden="true">
            <div className="orbit orbit-one" /><div className="orbit orbit-two" />
            <div className="disk"><div className="disk-inner"><div className="disk-core" /></div></div>
            <div className="orbit-label label-top">DEVICE SCAN <b>01</b></div>
            <div className="orbit-label label-bottom"><span /> LOCAL FIRST · READ ONLY</div>
          </div>
        </section>

        <div className="section-heading"><div><span className="eyebrow">HARDWARE INVENTORY</span><h3>Dispositivos de armazenamento</h3></div><span className="count">{devices.length} encontrados</span></div>

        {error && <div className="notice error"><strong>Não foi possível detectar os discos</strong><span>{error}</span><small>Confira as permissões do sistema e tente novamente.</small></div>}
        {!error && devices.length === 0 && (
          <div className="empty-state">
            <div className="empty-icon">◫</div>
            <strong>{scanned ? "Nenhum disco compatível encontrado" : "Aguardando detecção"}</strong>
            <span>{scanned ? "Conecte um dispositivo de armazenamento e tente novamente." : "A detecção é somente leitura. Nenhum dado será alterado."}</span>
          </div>
        )}
        {devices.length > 0 && (
          <div className="device-grid">
            {devices.map((device) => (
              <article className="device-card" key={device.stable_id ?? device.device_path}>
                <div className="device-card-top"><div className="drive-icon">▱</div><span className="tag">DETECTADO</span></div>
                <h4>{device.model}</h4>
                <div className="device-path">{device.device_path}</div>
                <div className="device-meta"><span>CAPACIDADE</span><b>{formatBytes(device.capacity_bytes)}</b></div>
                <div className="device-meta"><span>SERIAL</span><b>{device.serial || "Não informado"}</b></div>
                <div className="device-footer"><span className="status-dot" /> Identificado · leitura somente</div>
              </article>
            ))}
          </div>
        )}

        <section className="safety-note"><span className="shield">◇</span><div><strong>Proteção de dados em primeiro lugar</strong><p>Esta versão realiza apenas descoberta de hardware. Testes de superfície, clonagem, restauração e gravação ainda não estão habilitados.</p></div></section>
        <footer>DECODE DISK DOCTOR <span>BUILD 0.1.0 · DECODE ANALYTICS © 2026</span></footer>
      </section>
    </main>
  );
}
