Execute calculation with external programs

**Example 1: Optimize structure with xtb**

```yaml
run:
- run:
    with: Calculation
    # For each structure, a folder with name of the structure will be created under `A1_xtb`
    working_directory: A1_xtb
    # Specify the format of the input file
    pre_format:  
      format: xyz # XYZ format
      openbabel: true # Standardizing format with OpenBabel
      export_map: true # Export namespace in a JSON file. For example, input.map.json will be created for input.xyz
    # Filename of the input file
    pre_filename: input.xyz
    # Using serial mode to execute the external program
    # For programs that can not take advantages of parallel calculation, omit this field or set to false.
    serial_mode: true
    # The program name, it should be put in path or give a absolute path
    program: xtb
    # CLI arguments to the program
    args: [input.xyz, --gfn, '2', --opt]
    # Environment variables added or modified for the program. The key and values should be string
    envs:
      OMP_NUM_THREAD: '16'
    # The output file format and filename of the calculation program. LME will read them as the updated structure.
    post_file: [xyz, xtbopt.xyz]
    # Redirect stdout to a file
    stdout: xtb.out
    # Redirect stderr to a file
    stderr: xtb.err
```

**Example 2: Output structures in mol2 format**

```yaml
- run:
    with: Calculation
    # The output directory is `output`, and each structure in 
    # workspace will be put in a sub-directory
    working_directory: output
    # Specify the format
    pre_format:  
      format: mol2 # mol2 format
      openbabel: true # Standardizing format with OpenBabel
    # Filename of the file
    pre_filename: A2.mol2
    # All other fields can be ignored
```
