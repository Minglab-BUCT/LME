For each structure in workspace, add layers in this runner respectively. 

For example, there are structure A, B, C and D in current workspace, and the mirror isomer also need to be created, use `DistributeLayers`:

```yaml
run:
    with: DistributeLayers
    # R configuration is current configuration, add a transparent layer on it.
    R:
        type: Tranparent
    # S configuration should be create by a mirror operation, add a mirror layer on it.
    S:
        type: Mirror
        select: null
```

And then structure `A_R`, `A_S`, `B_R`, `B_S`, `C_R`, `C_S`, `D_R`, `D_S` will be created.

![The diagram of the DistributeLayers example][distributelayers]
