// Generated schema identity compatibility aliases.
export const schemaUriAliases: Readonly<Record<string, string>> = {
  "https://netcircuit.local/contracts/circuit/1.x/circuit.schema.json": "https://cirnet.local/contracts/circuit/1.x/circuit.schema.json",
  "https://netcircuit.local/contracts/circuit/1.x/module.schema.json": "https://cirnet.local/contracts/circuit/1.x/module.schema.json",
  "https://netcircuit.local/contracts/circuit/1.x/connection.schema.json": "https://cirnet.local/contracts/circuit/1.x/connection.schema.json",
  "https://netcircuit.local/contracts/memory/1.0/memory-image.schema.json": "https://cirnet.local/contracts/memory/1.0/memory-image.schema.json",
  "https://netcircuit.local/contracts/physical-lab/1.0/lab.schema.json": "https://cirnet.local/contracts/physical-lab/1.0/lab.schema.json",
  "https://netcircuit.local/contracts/design/visual/1.0/schema.json": "https://cirnet.local/contracts/design/visual/1.0/schema.json",
  "https://netcircuit.local/contracts/public/common/1.0/schema.json": "https://cirnet.local/contracts/public/common/1.0/schema.json",
  "https://netcircuit.local/contracts/public/api/1.0/schema.json": "https://cirnet.local/contracts/public/api/1.0/schema.json",
  "https://netcircuit.local/contracts/public/realtime/1.0/schema.json": "https://cirnet.local/contracts/public/realtime/1.0/schema.json",
  "https://netcircuit.local/contracts/public/auth/1.0/schema.json": "https://cirnet.local/contracts/public/auth/1.0/schema.json",
  "https://netcircuit.local/contracts/public/projects/1.0/schema.json": "https://cirnet.local/contracts/public/projects/1.0/schema.json",
  "https://netcircuit.local/contracts/public/assignments/1.0/schema.json": "https://cirnet.local/contracts/public/assignments/1.0/schema.json",
  "https://netcircuit.local/contracts/public/history/1.0/schema.json": "https://cirnet.local/contracts/public/history/1.0/schema.json",
  "https://netcircuit.local/contracts/public/instrument-control/1.0/schema.json": "https://cirnet.local/contracts/public/instrument-control/1.0/schema.json",
  "https://netcircuit.local/contracts/design/signal-domains/1.0/schema.json": "https://cirnet.local/contracts/design/signal-domains/1.0/schema.json",
  "https://netcircuit.local/contracts/capture/ncap/2.0/schema.json": "https://cirnet.local/contracts/capture/ncap/2.0/schema.json",
  "https://netcircuit.local/contracts/design/flexible-mount/1.0/schema.json": "https://cirnet.local/contracts/design/flexible-mount/1.0/schema.json",
  "https://netcircuit.local/contracts/circuit/1.2/schema.json": "https://cirnet.local/contracts/circuit/1.2/schema.json",
  "https://netcircuit.local/contracts/public/placement/2.0/schema.json": "https://cirnet.local/contracts/public/placement/2.0/schema.json",
  "https://netcircuit.local/contracts/public/assignments/2.0/schema.json": "https://cirnet.local/contracts/public/assignments/2.0/schema.json",
  "https://netcircuit.local/contracts/design/flexible-mount/1.1/schema.json": "https://cirnet.local/contracts/design/flexible-mount/1.1/schema.json",
  "https://netcircuit.local/contracts/design/package-shape/1.0/schema.json": "https://cirnet.local/contracts/design/package-shape/1.0/schema.json",
  "https://netcircuit.local/contracts/design/label-properties/1.0/schema.json": "https://cirnet.local/contracts/design/label-properties/1.0/schema.json",
  "https://netcircuit.local/contracts/design/board-insertion/1.0/schema.json": "https://cirnet.local/contracts/design/board-insertion/1.0/schema.json",
  "https://netcircuit.local/contracts/design/breadboard-mechanics/1.0/schema.json": "https://cirnet.local/contracts/design/breadboard-mechanics/1.0/schema.json"
}
export function canonicalSchemaUri(uri: string): string {
  const index = uri.indexOf('#')
  const document = index < 0 ? uri : uri.slice(0, index)
  return (schemaUriAliases[document] ?? document) + (index < 0 ? '' : uri.slice(index))
}
