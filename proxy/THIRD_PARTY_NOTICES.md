# Proxy image dependencies

The image pins Debian bookworm-slim by OCI digest, Python3 3.11.2-1+b1,
iptables 1.8.9-2 and ca-certificates 20250419~deb12u1. All newly installed transitive package versions are fixed in
`debian-packages-aarch64.lock`; base packages are bound by the OCI digest.
The current reviewed platform is Linux aarch64; other architectures fail closed
until an equivalent reviewed lock and worker tool-capture evidence are supplied.
Debian copyright/license notices remain in `/usr/share/doc/*/copyright`.
Python is distributed under PSF and associated licenses; iptables under GPL-2.0;
Debian ca-certificates packaging under its recorded component licenses. The
platform proxy and namespace-owner scripts are Apache-2.0 original code.
