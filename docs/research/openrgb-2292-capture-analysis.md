# OpenRGB issue #2292 capture analysis

Status: **CONFIRMED from the public packet captures**, except where explicitly marked as interpretation.

## Method

The `kd3b-capture` research tool parses pcapng + USBPcap records without touching hardware. A GitHub Actions research run downloaded the public captures linked from OpenRGB issue #2292, extracted non-empty OUT payloads for USB endpoint number 3, and generated raw byte diffs.

Research run: `31961928423`.

Every non-descriptor capture analyzed here contains only 256-byte endpoint-3 OUT payloads. The only five-byte payload headers observed are the already-documented Direct RGB headers:

```text
Packet A: 08 07 00 00 00
Packet B: 08 07 00 01 00
```

No separate mode/effect/brightness command packet was found in these endpoint-3 OUT streams.

## Capture hashes

```text
RED.pcapng             4f1634ba7008eba9a230b0be86e827c450005ba907a9c99ee4fce9275a1903eb
GREEN.pcapng           d96a3853b8253e066393082e4c47ea981d009b63e0e750acb62e0796e9e3ff78
BLUE.pcapng            e73368d5bdf67ac3297df3f4a671163bdce3f536315a861337bb2be7d0e94d82
OFF.pcapng             9954c69f25f6ca21aa0d0825154865c55d2b7b5530ee4e88394ab62b13cbecc7
WAVE.pcapng            4b8d1acb2d7c2822480e48bbe147097c028e2884baa9137e30846fded45ce821
ConicBand.pcapng       8991cb6b9473f5e00f64bc430df6faadb4c8b44e372d0d22206b90f4fc31bb09
Spiral.pcapng          be4d9f316cf57130341abb62214aac1aec8510c83bf81635d2d2f7081b67af74
Cycle.pcapng           2360be3f9b28771ade2da07f62ba674bb7a3c13945d3218cd25274bf7366730c
LinearWave.pcapng      bd8c3a817308b7aab0af76d6936596af2f488f977bed4fb6c4e008def102c00a
Ripple.pcapng          26591ccc1c6aafde1fc1f56acb7296d8f1710b5a982c3650fd38a9c93bc07fc7
Breathing.pcapng       bbd6661b1c1adf5405454ca0b4e6e317441b4a2eaf8cec68965fb5eb8a82b007
Rain.pcapng            a4c4b8cb82fc4692d75d3eda90164261b32da29a996be4c4dacf8e1429203a50
Fire.pcapng            16a6052e0f4a9c3c87582281fc883aa9e9e3ab9a09daa067ad418eab3aaaaad4
Trigger.pcapng         ce8be748dfbdf84e9854cd9e2aa29fe15cf744c893828f25717fe33e8bf136e2
Brightness0-100.pcapng c6c630155593b8d5c96d7715b6e61ac65232d272d2e7b3d586e56c9bc51e1463
```

## Direct RGB frame evidence

The payload streams alternate the two known Direct RGB packet types. Complete adjacent pairs in these captures appear in capture order Packet B then Packet A. This is an observation about these USBPcap recordings, not a reason to change the application's validated A-then-B write order: the Linux hardware test already proved A then B produces the intended result on the target keyboard.

The solid-color captures contain stable full-key frames:

| Capture | Stable decoded frame |
| --- | --- |
| RED | all 87 keys `255,0,0` |
| GREEN | all 87 keys `4,255,0` |
| BLUE | all 87 keys `0,17,255` |
| OFF | all 87 keys `0,0,0` after the transition |

The small non-zero red/green components in the GREEN/BLUE recordings are capture facts. They most likely reflect the color selected in the OEM UI; they are not protocol constants.

## Brightness capture

`Brightness0-100.pcapng` contains 1062 endpoint-3 OUT payloads, or 531 adjacent A/B frame pairs. The decoded frames are uniform grayscale frames whose channel values rise monotonically from `0` to `255` during the recording. Stable observed values include:

```text
0, 2, 5, 7, 10, 12, 17, 22, 25, 28, 30, 38, 45, 61, 66,
76, 86, 94, 107, 119, 127, 155, 160, 173, 186, 201, 221, 232,
244, 247, 255
```

There is no separate brightness field or brightness command in the extracted endpoint-3 OUT stream. The evidence therefore supports host-side brightness scaling of Direct RGB channel values for this recording. It does **not** prove that the keyboard has no undocumented onboard brightness command reachable through some other transaction or interface.

## Named effect captures

Each named effect is a sequence of ordinary Direct RGB frames:

| Capture | Decoded adjacent frame pairs | Frame characteristic |
| --- | ---: | --- |
| WAVE | 122 | all 87 keys lit; spatial colors change every frame |
| ConicBand | 102 | all 87 keys lit; spatial colors change every frame |
| Spiral | 99 | all 87 keys lit; spatial colors change almost every frame |
| Cycle | 161 | each frame is uniform across all 87 keys; color changes over time |
| LinearWave | 249 | moving sparse band; average about 17.5 lit keys |
| Ripple | 195 | moving sparse/radial pattern; average about 26.6 lit keys |
| Breathing | 660 | uniform frames across the keyboard; intensity/color changes over time |
| Rain | 133 | sparse drops; average about 2.8 lit keys, maximum 5 in this recording |
| Fire | 421 | dense flickering field; average about 39.5 lit keys, maximum 80 |
| Trigger | 246 | mostly dark; at most one lit key in the decoded sample frames |

These characteristics are derived from the 87-key mapping already documented by this project.

## Consequence for implementation

The public captures do **not** provide evidence for a distinct "OEM onboard effect packet" protocol. They provide strong evidence that the captured OEM application renders these named effects on the host and streams them through the same Direct RGB frame format that is already working on Linux.

Accordingly:

1. Keep the verified Direct RGB transport as the hardware backend.
2. Implement brightness as a host-side frame transform for software-rendered Direct RGB.
3. Implement the named effects in the host software effect engine.
4. Treat persistent/onboard effect control as still unknown unless a different capture shows a separate command path.
5. Do not invent persistent packets from the effect names.

This changes the roadmap: software effect rendering can proceed immediately; persistent/onboard protocol research remains a separate evidence-gated task.
