Append Layers to all structures in the workspace.

This runner takes a list of layers as input.

Example:

```yaml
run:
    with: AppendLayers
    layers:
    - type: IdMap
        Ru: 0
        H1: 1
        H2: 25
        N1: 2
        P1: 11
        R11: 12
        R12: 13
        P2: 14
        R21: 16
        R22: 15
        C1: 17
        C1Hon: 18
        C1Hdown: 19
        C2: 20
        C2Hon: 21
        C2Hdown: 22
        C3: 3
        C4: 4
    - type: GroupMap
        groups:
        - [RuN, [Ru, N1]]
        - [CO, [23,24]]
        - [bone, {
        includes: [null],
        excludes: [[R11, R12, R21, R22]]
        }]
```

![The diagram of AppendLayers][appendlayers]