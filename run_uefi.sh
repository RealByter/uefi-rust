#!/bin/bash

mkdir -p esp/efi/boot

cp target/x86_64-unknown-uefi/debug/uefi-rust.efi esp/efi/boot/bootx64.efi

qemu-system-x86_64 \
    -device virtio-rng-pci \
    -machine q35 \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:esp \
    -net none \
    -serial stdio