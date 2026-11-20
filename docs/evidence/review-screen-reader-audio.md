# Screen-reader audio prerequisite

Scope: audible output under Kata `q72g` and `9s44`.
Date verified: November 20, 2026.

## Actual runner result

The [disposable probe](https://github.com/2389-research/spaceport/actions/runs/44756776215) runs exact source `5c0fc1bb42a8783d85c30482553b28b73e305efe`.
Its [job](https://github.com/2389-research/spaceport/actions/runs/44756776215/job/136358454558) fails at actual speech playback.
The image is `windows-2025-vs2026`, version `20261116.315.1`.
The operating system reports Windows Server 2025, build 26100.

Both audio services report status 4.
The Win32 sound-device query returns an empty array.
Microsoft David Desktop and Microsoft Zira Desktop are installed and enabled.
`SetOutputToDefaultAudioDevice` returns, but `Speak` raises `AudioException` with audio-device error `0x2`.
The [structured receipt](review-squad/windows-native/speech-prerequisite-5c0fc1bb4.json) preserves the observations and source.

This verifies a failed default playback attempt on this exact hosted image.
It does not prove that every Windows runner lacks audio.
The device inventory does not enumerate active WASAPI endpoints.
No screen reader runs, and no audio is recorded or heard.
The accessibility finding stays open.

## Qualification route

First qualify an actual render endpoint on an isolated Windows desktop.
Keep its source, image, and audio-device observations with the test receipt.
Use a pinned portable NVDA copy with private configuration and its built-in eSpeak NG voice.
Keep Spaceport and NVDA at the same integrity level.
Disable typed-character and typed-word speech in the private reader configuration.
Produce an output-only phrase through the existing private Herdr fixture.
Do not invoke a read-current-line command or inject speech directly.

Record only the owned NVDA process tree through process-scoped WASAPI capture.
Keep a bounded WAV, its hash, timing, reader settings, exact source, and cleanup receipts.
Listen to the WAV and verify the authored phrase.
Nonzero audio, speech text, and UIA events alone do not qualify audible output.

## Primary references

- [Default speech output API](https://learn.microsoft.com/en-us/dotnet/api/system.speech.synthesis.speechsynthesizer.setoutputtodefaultaudiodevice?view=netframework-4.8.1) supplies the probe's render path.
- [Win32 sound-device inventory](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-sounddevice) describes the hardware query's scope.
- [Microsoft process-loopback sample](https://learn.microsoft.com/en-us/samples/microsoft/windows-classic-samples/applicationloopbackaudio-sample/) records one process tree to WAV.
  It requires build 20348 or later and returns silence when that tree renders no audio.
- [NVDA guide](https://api.nvaccess.org/documentation/userGuide.html) documents portable operation, private configuration, eSpeak NG, and output-device selection.
