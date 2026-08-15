#!/usr/bin/env fish

set -l root (realpath (dirname (status filename))/..)
set -l out "$root/captures/raw"
mkdir -p "$out"

set -l base "https://gitlab.com/CalcProgrammer1/OpenRGB/-/uploads"

set -l items \
"b506fc715aa1b396c6e4fe8d5e1b6882/KD3Brev.2_ReportDescriptor.pcapng" \
"9546918bdf1f7856bbbe84fa83b7526e/RED.pcapng" \
"c18288941db8153880846f249506697f/GREEN.pcapng" \
"a33c0543db2f616555b32dd77e52f739/BLUE.pcapng" \
"4647f96efbab6fb1a71490529554e002/OFF.pcapng" \
"938ee6bcb4d5360860b56bb92d9f89ba/WAVE.pcapng" \
"ad6de23e54909f06f66f45c1b0a72b4a/ConicBand.pcapng" \
"a6887a16fd0a8c42dcea68f265bbff25/Spiral.pcapng" \
"0659c8660ac44704ad17d08fa56a3461/Cycle.pcapng" \
"da4cdf4b6eb048cb373c50d8ce9da4b4/LinearWave.pcapng" \
"04ff33aafa5c8f574184010defbb0199/Ripple.pcapng" \
"3bfa2ce26f3ce3726bf452bbaedc74dd/Breathing.pcapng" \
"17530203343cb6b5218121bb4d7bc04a/Rain.pcapng" \
"44b9d4001d892e3281ac6f11feef6048/Fire.pcapng" \
"45ee4807ec0ecc124b004fd4b07c8a3f/Trigger.pcapng" \
"c8b47fbae59feb568c1541f006e147ae/Brightness0-100.pcapng"

for item in $items
    set -l name (basename "$item")
    echo "Fetching $name"
    curl -fL "$base/$item" -o "$out/$name"; or exit 1
end

echo
sha256sum "$out"/*.pcapng | tee "$out/SHA256SUMS"
