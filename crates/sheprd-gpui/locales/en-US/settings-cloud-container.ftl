# Settings > Cloud devices > Container. Engine names (Docker, Podman), their
# versions, and container names stay as they are.

settings-cloud-container-note = Existing saved containers on this computer's Docker or Podman engine. New container creation is unavailable.
settings-cloud-container-checking = Checking for Docker or Podman…
settings-cloud-container-engine-running = { $engine } { $version } is running.

## The engine's fields, each a label and, when not a default value, its placeholder.

settings-cloud-container-engine = Engine
# "docker" and "podman" are what the field takes; keep them as they are.
settings-cloud-container-engine-placeholder = docker or podman; the first one installed if empty

## New container and Containers.

settings-cloud-container-devices = Containers
settings-cloud-container-devices-note = Remove forgets the saved device. The container and its files remain on the engine; manage them outside Spaceport.
# A container's row: the engine it runs on, then its state.
settings-cloud-container-running = { $engine } · running
settings-cloud-container-stopped = { $engine } · stopped
settings-cloud-container-missing = { $engine } · missing from the engine
settings-cloud-container-unreachable = { $engine } · engine not reachable
