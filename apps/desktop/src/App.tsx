import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type Device = {
  device_path: string;
  stable_id: string | null;
  serial: string | null;
  model: string;
  capacity_bytes: number;
};

type StorageEndpoint = { kind: "BlockDevice" | "ImageFile"; path: string; stable_identity: string | null; capacity_bytes: number; is_system: boolean; is_mounted: boolean; is_read_only: boolean };

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
  const [cloneSource, setCloneSource] = useState("");
  const [cloneTarget, setCloneTarget] = useState("");
  const [clonePhrase, setClonePhrase] = useState("");
  const [clonePhraseInput, setClonePhraseInput] = useState("");
  const [imageSource, setImageSource] = useState("");
  const [imageTarget, setImageTarget] = useState("");
  const [imagePath, setImagePath] = useState("");
  const [restoreSourceDisk, setRestoreSourceDisk] = useState("");
  const [restoreTargetDisk, setRestoreTargetDisk] = useState("");
  const [restoreImagePath, setRestoreImagePath] = useState("");
  const [restoreImageBytes, setRestoreImageBytes] = useState("");
  const [restorePhrase, setRestorePhrase] = useState("");
  const [restorePhraseInput, setRestorePhraseInput] = useState("");
  const [operationMessage, setOperationMessage] = useState("");
  const [operationBusy, setOperationBusy] = useState(false);

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


  function endpoint(device: Device, kind: StorageEndpoint["kind"] = "BlockDevice", path = device.device_path): StorageEndpoint {
    return { kind, path, stable_identity: device.stable_id, capacity_bytes: device.capacity_bytes,
      is_system: false, is_mounted: false, is_read_only: false };
  }

  async function prepareClone() {
    const source = devices.find((d) => d.device_path === cloneSource);
    const target = devices.find((d) => d.device_path === cloneTarget);
    if (!source || !target) { setOperationMessage("Selecione os discos de origem e destino."); return; }
    try {
      setOperationBusy(true); setOperationMessage("");
      const phrase = await invoke<string>("operation_confirmation_phrase", {
        kind: "BlockClone", source: endpoint(source), destination: endpoint(target), requestedBytes: source.capacity_bytes
      });
      setClonePhrase(phrase); setClonePhraseInput("");
      setOperationMessage("Pré-validação concluída. Confira os discos e digite a frase exata para autorizar a clonagem.");
    } catch (cause) { setOperationMessage(String(cause)); }
    finally { setOperationBusy(false); }
  }

  async function executeClone() {
    const source = devices.find((d) => d.device_path === cloneSource);
    const target = devices.find((d) => d.device_path === cloneTarget);
    if (!source || !target || !clonePhrase || clonePhraseInput !== clonePhrase) {
      setOperationMessage("A confirmação não corresponde à operação preparada."); return;
    }
    try {
      setOperationBusy(true); setOperationMessage("Clonagem em andamento. Não desconecte os discos.");
      const result = await invoke<{ bytes_copied: number; sha256: string }>("clone_physical_disk", {
        source: endpoint(source), destination: endpoint(target), requestedBytes: source.capacity_bytes,
        confirmationPhrase: clonePhraseInput
      });
      setOperationMessage(`Clonagem concluída: ${formatBytes(result.bytes_copied)} · SHA-256 ${result.sha256}`);
      setClonePhrase(""); setClonePhraseInput("");
    } catch (cause) { setOperationMessage(`A clonagem não foi concluída: ${String(cause)}`); }
    finally { setOperationBusy(false); }
  }

  async function createImage() {
    const source = devices.find((d) => d.device_path === imageSource);
    const target = devices.find((d) => d.device_path === imageTarget);
    if (!source || !target || !imagePath.trim()) {
      setOperationMessage("Selecione origem, destino, e informe o caminho completo do arquivo de imagem."); return;
    }
    try {
      setOperationBusy(true); setOperationMessage("Criando imagem raw. A operação pode levar bastante tempo.");
      const result = await invoke<{ bytes_processed: number; sha256: string }>("create_disk_image", {
        source: endpoint(source), destination: endpoint(target), imagePath,
        capacityBytes: source.capacity_bytes
      });
      setOperationMessage(`Imagem criada: ${formatBytes(result.bytes_processed)} · SHA-256 ${result.sha256}`);
    } catch (cause) { setOperationMessage(`Falha ao criar imagem: ${String(cause)}`); }
    finally { setOperationBusy(false); }
  }

  async function prepareRestore() {
    const sourceDisk = devices.find((d) => d.device_path === restoreSourceDisk);
    const targetDisk = devices.find((d) => d.device_path === restoreTargetDisk);
    const bytes = Number(restoreImageBytes);
    if (!sourceDisk?.stable_id || !targetDisk || !restoreImagePath.trim() || !Number.isSafeInteger(bytes) || bytes <= 0) {
      setOperationMessage("Informe o disco onde está a imagem, o destino, o caminho e o tamanho em bytes."); return;
    }
    const imageEndpoint: StorageEndpoint = { kind: "ImageFile", path: restoreImagePath, stable_identity: sourceDisk.stable_id,
      capacity_bytes: bytes, is_system: false, is_mounted: false, is_read_only: true };
    try {
      setOperationBusy(true); setOperationMessage("");
      const phrase = await invoke<string>("operation_confirmation_phrase", {
        kind: "Restore", source: imageEndpoint, destination: endpoint(targetDisk), requestedBytes: bytes
      });
      setRestorePhrase(phrase); setRestorePhraseInput("");
      setOperationMessage("Pré-validação concluída. Restaurar apagará os dados do disco de destino. Digite a frase exata para continuar.");
    } catch (cause) { setOperationMessage(String(cause)); }
    finally { setOperationBusy(false); }
  }

  async function executeRestore() {
    const sourceDisk = devices.find((d) => d.device_path === restoreSourceDisk);
    const targetDisk = devices.find((d) => d.device_path === restoreTargetDisk);
    const bytes = Number(restoreImageBytes);
    if (!sourceDisk?.stable_id || !targetDisk || !restorePhrase || restorePhraseInput !== restorePhrase) {
      setOperationMessage("A confirmação não corresponde à restauração preparada."); return;
    }
    const imageEndpoint: StorageEndpoint = { kind: "ImageFile", path: restoreImagePath, stable_identity: sourceDisk.stable_id,
      capacity_bytes: bytes, is_system: false, is_mounted: false, is_read_only: true };
    try {
      setOperationBusy(true); setOperationMessage("Restauração em andamento. O disco de destino será sobrescrito.");
      const result = await invoke<{ bytes_processed: number; sha256: string }>("restore_disk_image", {
        imageSource: imageEndpoint, destination: endpoint(targetDisk), imagePath: restoreImagePath,
        imageBytes: bytes, confirmationPhrase: restorePhraseInput
      });
      setOperationMessage(`Restauração concluída: ${formatBytes(result.bytes_processed)} · SHA-256 ${result.sha256}`);
      setRestorePhrase(""); setRestorePhraseInput("");
    } catch (cause) { setOperationMessage(`Restauração não concluída: ${String(cause)}`); }
    finally { setOperationBusy(false); }
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

        <section className="safety-note" style={{ marginTop: 28, display: "block" }}>
          <span className="eyebrow blue">OPERAÇÕES REAIS · LINUX</span>
          <h3 style={{ margin: "8px 0" }}>Clonagem física e imagens de disco</h3>
          <p>As operações abaixo usam leitura/gravação real de dispositivos. Use somente discos de teste ou cópias. O destino de clonagem e restauração será sobrescrito.</p>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit,minmax(230px,1fr))", gap: 16, marginTop: 18 }}>
            <div>
              <strong>Clonar disco</strong>
              <label style={{ display: "block", marginTop: 10 }}>Origem</label>
              <select value={cloneSource} onChange={(e) => { setCloneSource(e.target.value); setClonePhrase(""); }} style={{ width: "100%", padding: 10 }}>
                <option value="">Selecione a origem</option>{devices.map((d) => <option key={d.device_path} value={d.device_path}>{d.device_path} · {d.model}</option>)}
              </select>
              <label style={{ display: "block", marginTop: 10 }}>Destino (será apagado)</label>
              <select value={cloneTarget} onChange={(e) => { setCloneTarget(e.target.value); setClonePhrase(""); }} style={{ width: "100%", padding: 10 }}>
                <option value="">Selecione o destino</option>{devices.map((d) => <option key={d.device_path} value={d.device_path}>{d.device_path} · {d.model}</option>)}
              </select>
              <button className="secondary" disabled={operationBusy || !devices.length} onClick={prepareClone} style={{ marginTop: 12 }}>Preparar clonagem segura</button>
              {clonePhrase && <>
                <p><small>Digite exatamente: <b>{clonePhrase}</b></small></p>
                <input value={clonePhraseInput} onChange={(e) => setClonePhraseInput(e.target.value)} placeholder="Frase de confirmação" style={{ width: "100%", padding: 10 }} />
                <button className="primary" disabled={operationBusy || clonePhraseInput !== clonePhrase} onClick={executeClone} style={{ marginTop: 10 }}>Clonar agora</button>
              </>}
            </div>
            <div>
              <strong>Criar imagem raw</strong>
              <label style={{ display: "block", marginTop: 10 }}>Disco de origem</label>
              <select value={imageSource} onChange={(e) => setImageSource(e.target.value)} style={{ width: "100%", padding: 10 }}>
                <option value="">Selecione a origem</option>{devices.map((d) => <option key={d.device_path} value={d.device_path}>{d.device_path} · {d.model}</option>)}
              </select>
              <label style={{ display: "block", marginTop: 10 }}>Disco onde salvar a imagem</label>
              <select value={imageTarget} onChange={(e) => setImageTarget(e.target.value)} style={{ width: "100%", padding: 10 }}>
                <option value="">Selecione o armazenamento de destino</option>{devices.map((d) => <option key={d.device_path} value={d.device_path}>{d.device_path} · {d.model}</option>)}
              </select>
              <label style={{ display: "block", marginTop: 10 }}>Caminho completo do arquivo</label>
              <input value={imagePath} onChange={(e) => setImagePath(e.target.value)} placeholder="/mnt/backup/disco.img" style={{ width: "100%", padding: 10 }} />
              <button className="secondary" disabled={operationBusy} onClick={createImage} style={{ marginTop: 12 }}>Criar imagem</button>
            </div>
            <div>
              <strong>Restaurar imagem</strong>
              <label style={{ display: "block", marginTop: 10 }}>Armazenamento onde está a imagem</label>
              <select value={restoreSourceDisk} onChange={(e) => setRestoreSourceDisk(e.target.value)} style={{ width: "100%", padding: 10 }}>
                <option value="">Selecione o armazenamento</option>{devices.map((d) => <option key={d.device_path} value={d.device_path}>{d.device_path} · {d.model}</option>)}
              </select>
              <label style={{ display: "block", marginTop: 10 }}>Disco de destino (será sobrescrito)</label>
              <select value={restoreTargetDisk} onChange={(e) => { setRestoreTargetDisk(e.target.value); setRestorePhrase(""); }} style={{ width: "100%", padding: 10 }}>
                <option value="">Selecione o destino</option>{devices.map((d) => <option key={d.device_path} value={d.device_path}>{d.device_path} · {d.model}</option>)}
              </select>
              <label style={{ display: "block", marginTop: 10 }}>Arquivo de imagem</label>
              <input value={restoreImagePath} onChange={(e) => setRestoreImagePath(e.target.value)} placeholder="/mnt/backup/disco.img" style={{ width: "100%", padding: 10 }} />
              <label style={{ display: "block", marginTop: 10 }}>Tamanho da imagem em bytes</label>
              <input value={restoreImageBytes} onChange={(e) => setRestoreImageBytes(e.target.value)} placeholder="Ex.: 500107862016" style={{ width: "100%", padding: 10 }} />
              <button className="secondary" disabled={operationBusy} onClick={prepareRestore} style={{ marginTop: 12 }}>Preparar restauração</button>
              {restorePhrase && <>
                <p><small>Digite exatamente: <b>{restorePhrase}</b></small></p>
                <input value={restorePhraseInput} onChange={(e) => setRestorePhraseInput(e.target.value)} placeholder="Frase de confirmação" style={{ width: "100%", padding: 10 }} />
                <button className="primary" disabled={operationBusy || restorePhraseInput !== restorePhrase} onClick={executeRestore} style={{ marginTop: 10 }}>Restaurar agora</button>
              </>}
            </div>
          </div>
          {operationMessage && <div className="notice" style={{ marginTop: 18 }}><strong>Resultado da operação</strong><span>{operationMessage}</span></div>}
        </section>

        <section className="safety-note"><span className="shield">◇</span><div><strong>Proteção de dados em primeiro lugar</strong><p>Descoberta de hardware e leitura SMART são somente leitura. Clonagem, imagem raw e restauração agora têm motores Linux com confirmação vinculada ao plano. Use apenas mídia de teste até validar o fluxo com hardware real.</p></div></section>
        <footer>DECODE DISK DOCTOR <span>BUILD 0.1.0 · DECODE ANALYTICS © 2026</span></footer>
      </section>
    </main>
  );
}
