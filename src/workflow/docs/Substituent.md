Change substituent on specified positions.

A substituent process takes 2 structures and handle 4 atoms:

- origin structure
  - atom A: the atom connect to the substituent group on the origin structure bone
  - atom B: the leading atom to tell the `Substituent` runner the orientation of the atom
- substituent group structure
  - atom C: the leading atom to control the direction of orientation of the substituent structure
  - atom D: the atom connect to the main structure on the substituent group

![The diagram of the substituent process][substituent]

This runner takes two arguments:

- address
- file_pattern

`address` is a map that describe the position of substituent, the keys are the position name and values are the atoms to be replaced. For example, `R21g: [P2, R21]` means replace the `R21` atom connect to `P2` atom, and the atoms added during the substituent will be put in the group named `R21g`. The changes of all substitution sites in a single Substituent Runner are synchronized, e.g. when R21 changes to Ph, R22 also changes to Ph, and when R21 changes to Me, R22 is also Me. If it is desirable to produce a combinatorial result, the different sites can be substituted individually in consecutive Substituent Runners.

`file_pattern` is a list of file patterns for loading the substituent file. For example, `- substituent/*.lme` means use all files with extension name `lme` as substituents. To explicit declare which files are used, you can give a full by YAML list syntax like:

```yaml
file_pattern: 
    - substituent/Ph.lme
    - substituent/Me.lme
```

Example:

```yaml
run:
    with: Substituent
    address:
        R21g: [P2, R21]
        R22g: [P2, R22]
    file_pattern: 
        - substituent/*.lme
```
