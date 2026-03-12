export const posManifestPhoenix2026 = {
    id: "IDEIS-POS-PHX-2026-001",
    name: "Phoenix SmartCity POS QPU.Datashard",
    repos: [
        "git://city/phx/qpu_datashard/pos",
        "git://city/phx/qpu_datashard/devtunnel",
        "git://city/phx/qpu_datashard/searchtrace",
    ],
    jurisdictions: ["US-AZ-PHX"],
    actions: ["POS_SALE", "INVENTORY_SYNC", "DEV_TUNNEL_OPEN", "SEARCHTRACE_AUDIT"],
    checksum: "",
};

export function sealManifest(manifest, hashFn) {
    const flat = [
        manifest.id,
        manifest.name,
        manifest.repos.join(","),
        manifest.jurisdictions.join(","),
        manifest.actions.join(","),
    ].join("|");
    manifest.checksum = hashFn(flat);
    return manifest;
}
