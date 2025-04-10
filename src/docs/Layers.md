Layers are the basic storage unit of the LME. Each structure in LME can be represented in serveral layers.

In a LME build process, structures are constructed from the given base structure, and then modified by the given layers. With different layer stacks, 
different structures with same basement will be created.

There are two classes of layers: fill layers and function layers. Fill layers contains partial structures with 3D coordinates, they will directly modify the 
structure with given data. Function layers contains a function with arguments, they will modify the structure with more complex logic.

![Layers in LME][layers_in_lme]

The 