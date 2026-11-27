import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Device = {
  device_path: string;
  stable_id: string | null;
  serial: string | null;
  model: string;
  capacity_bytes: number;
};

type Measurement<T> =
  | { Measured: T }
  | { Unsupported: { reason: string } }
  | { Unavailable: { reason: string } };

type SmartSnapshot = {
  device: Device;
  captured_at: string;
  temperature_celsius: Measurement<number>;
  power_on_hours: Measurement<number>;
  reallocated_sector_count: Measurement<number>;
  pending_sector_count: Measurement<number>;
  overall_note: string;
};

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "Capacidade indisponível";
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit += 1;
  }
  return `${value.toFixed(value >= 100 ? 0 : 1)} ${units[unit]}`;
}

function measurementLabel<T>(measurement: Measurement<T>, suffix = ""): string {
  if ("Measured" in measurement) return `${measurement.Measured}${suffix}`;
  if ("Unsupported" in measurement) return "Não suportado";
  return "Indisponível";
}

export default function App() {
  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [scanned, setScanned] = useState(false);
  const [smartData, setSmartData] = useState<Record<string, SmartSnapshot>>({});
  const [smartErrors, setSmartErrors] = useState<Record<string, string>>({});
  const [smartLoading, setSmartLoading] = useState<string | null>(null);

  async function scanDevices() {
    setLoading(true);
    setError("");
    setSmartData({});
    setSmartErrors({});
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

  async function inspectSmart(device: Device) {
    setSmartLoading(device.device_path);
    setSmartErrors((current) => ({ ...current, [device.device_path]: "" }));
    try {
      const result = await invoke<SmartSnapshot>("inspect_smart", { device });
      setSmartData((current) => ({ ...current, [device.device_path]: result }));
    } catch (cause) {
      setSmartErrors((current) => ({ ...current, [device.device_path]: String(cause) }));
    } finally {
      setSmartLoading(null);
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
            {devices.map((device) => {
              const smart = smartData[device.device_path];
              return (
                <article className="device-card" key={device.stable_id ?? device.device_path}>
                  <div className="device-card-top"><div className="drive-icon">▱</div><span className="tag">DETECTADO</span></div>
                  <h4>{device.model}</h4>
                  <div className="device-path">{device.device_path}</div>
                  <div className="device-meta"><span>CAPACIDADE</span><b>{formatBytes(device.capacity_bytes)}</b></div>
                  <div className="device-meta"><span>SERIAL</span><b>{device.serial || "Não informado"}</b></div>
                  <div className="device-footer"><span className="status-dot" /> Identificado · leitura somente</div>
                  <button className="secondary" onClick={() => inspectSmart(device)} disabled={smartLoading === device.device_path}>
                    {smartLoading === device.device_path ? "Consultando SMART..." : "Ler dados SMART"}
                  </button>
                  {smartErrors[device.device_path] && <div className="smart-error">{smartErrors[device.device_path]}</div>}
                  {smart && (
                    <div className="smart-panel">
                      <strong>LEITURA SMART</strong>
                      <div className="smart-row"><span>Temperatura</span><b>{measurementLabel(smart.temperature_celsius, " °C")}</b></div>
                      <div className="smart-row"><span>Horas ligado</span><b>{measurementLabel(smart.power_on_hours, " h")}</b></div>
                      <div className="smart-row"><span>Setores realocados</span><b>{measurementLabel(smart.reallocated_sector_count)}</b></div>
                      <div className="smart-row"><span>Setores pendentes</span><b>{measurementLabel(smart.pending_sector_count)}</b></div>
                      <p>{smart.overall_note}</p>
                    </div>
                  )}
                </article>
              );
            })}
          </div>
        )}

        <section className="safety-note"><span className="shield">◇</span><div><strong>Proteção de dados em primeiro lugar</strong><p>Descoberta de hardware e leitura SMART são somente leitura. Testes de superfície, clonagem, restauração e gravação ainda não estão habilitados.</p></div></section>
        <footer>DECODE DISK DOCTOR <span>BUILD 0.1.0 · DECODE ANALYTICS © 2026</span></footer>
      </section>
    </main>
  );
}
