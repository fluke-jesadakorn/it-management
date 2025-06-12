# Network Machine Share Discovery

This document outlines how to discover available machine shares on the local network using DNS Service Discovery.

## Command to Find SSH Services

To list all available SSH services with the 'vg-ph' prefix:

```bash
dns-sd -B _ssh._tcp . | grep vg-ph
```

## Available Machines

The following machines are available on the network:

```text
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-toom102
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-kung
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-tatty
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-toom105
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-baifern
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-yale
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-tossi
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-toom103
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-luan
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-Suriya
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-yanapath
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-tumm
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-pohn
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-pakbung
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-wutt
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-E
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-gift
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-beam
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-fon
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-nipat
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-com-103
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-weerayut
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-nutt
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-sina
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-Atom
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-seagame
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-gig
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-pat
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-doe
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-nok
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-fluke
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-mew
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-matthias
16:00:57.504  Add        3  14 local.               _ssh._tcp.           vg-ph-karn
16:00:57.504  Add        2  14 local.               _ssh._tcp.           vg-ph-kai
