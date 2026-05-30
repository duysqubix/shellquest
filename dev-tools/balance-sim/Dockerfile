# shellquest balance-sim image — DEV-ONLY (not the shipped game image).
#
# Purpose: filesystem isolation for the balance simulator. The harness runs
# ONE container per simulated character so each character gets a pristine,
# disposable filesystem (save data, zone dirs, HOME) that can't leak into
# another character's run.
#
# IMPORTANT: neither the `sq` binary nor the Python sim code is baked in here.
# They arrive as read-only bind mounts at `docker run` time (wired up in W2):
#   - sim code  -> /sim   (ro)
#   - sq binary -> /opt/sq (ro)
#   - rw shard  -> /out
# This image is just a thin python-slim runtime + the pre-created "zone"
# directories the simulator uses as `--cwd` targets.
#
# Zone danger is computed by `sq` via PATH SEGMENT matching (src/zones.rs
# has_segment splits on '/'), so the exact path segments below matter:
#   /zones/src              -> danger 2 (segment 'src', Source Sanctum)
#   /zones/node_modules     -> danger 5 (segment 'node_modules', the Abyss)
#   /tmp                    -> danger 3 (Wasteland; exists in base image)
#   /dev                    -> danger 4 (Device Caverns; exists in base image)
#   $HOME (/sim-home)       -> danger 1 (Home Village; matched exactly by sq)

FROM python:3.12-slim

# Image-default env. SQ_DEBUG is on because this image is dev-only diagnostics;
# SQ_NO_PACING strips the arena's 1.5s pacing so sims resolve instantly.
# RUST_BACKTRACE=full surfaces panics from the mounted sq binary.
# HOME is the danger-1 "Home Village" dir (the harness may override it per
# character at runtime, but the image default must exist).
ENV HOME=/sim-home \
    RUST_BACKTRACE=full \
    SQ_NO_PACING=1 \
    SQ_DEBUG=1

# Pre-create zone `--cwd` targets, mount points, and the default HOME.
#   /sim   -> sim code mounts here (ro)
#   /out   -> rw output shard lands here
#   /opt   -> sq binary mounts to /opt/sq (ro)
# /zones keeps fixture dirs visible when sim code is mounted over /sim.
# /tmp and /dev already exist in the base image; ensure /tmp for safety.
RUN mkdir -p \
        /sim \
        /zones/src \
        /zones/node_modules \
        /out \
        /opt \
        /sim-home \
        /tmp

WORKDIR /sim

# No sim ENTRYPOINT — the run command (e.g. `python /sim/simulator/container_main.py ...`)
# is supplied by `docker run` in W2. A plain interactive python is the default.
CMD ["python3"]
