use std::{
    collections::{BTreeMap, BTreeSet},
    f64::consts::PI,
    fmt::Display,
    ops::RangeInclusive,
};

use bincode::{Decode, Encode};
use nalgebra::{Isometry3, Point3, Translation3, Vector3};
use redb::Value;
use serde::{Deserialize, Serialize};

use crate::{
    chemistry::Atom3D,
    group_name::GroupName,
    sparse_molecule::{SparseAtomList, SparseMolecule},
    utils::geometric::axis_angle_for_b2a,
};

#[doc = include_str!("docs/Layers.md")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[serde(tag = "type")]
pub enum Layer {
    /// This layer does nothing. It can be used as a placeholder, 
    /// for example creating S configuration from R configuration structures:
    /// 
    /// ```yml
    /// - run:
    ///   with: DistributeLayers
    ///   R:
    ///     type: Transparent
    ///   S:
    ///     type: Mirror
    /// ```
    Transparent,
    Fill {
        data: SparseMolecule,
    },
    Insert {
        offset: usize,
        data: SparseMolecule,
    },
    Append {
        name: String,
        data: SparseMolecule,
    },
    /// Update atoms by a list. To remove an atom, use null
    /// 
    /// ```yml
    /// type: SetAtom
    /// atoms:
    /// - [1, { element: 17, position: [1.0, 0.0, 0.0], formal_charge: 0 }]
    /// - [2, { element: H, position: [3.0, 0.0, 0.0], formal_charge: 0.0 }]
    /// ```
    SetAtom {
        atoms: Vec<(SelectOne, Option<Atom3D>)>,
    },
    /// Update atom elements
    UpdateElement {
        elements: Vec<(SelectOne, usize)>,
    },
    /// Update charges 
    UpdateFormalCharge {
        charges: Vec<(SelectOne, f64)>,
    },
    AppendAtoms {
        atoms: Vec<Atom3D>,
    },
    SetBond {
        bonds: Vec<(SelectOne, SelectOne, f64)>,
    },
    IdMap(BTreeMap<String, SelectOne>),
    GroupMap {
        groups: Vec<(String, SelectMany)>,
    },
    SetCenter {
        select: SelectOne,
        #[serde(default)]
        #[bincode(with_serde)]
        center: Point3<f64>,
    },
    DirectionAlign {
        select: SelectOne,
        #[serde(default = "Vector3::x")]
        #[bincode(with_serde)]
        direction: Vector3<f64>,
    },
    XYAlign {
        o: SelectOne,
        x: SelectOne,
        y: SelectOne,
        #[serde(default)]
        select: SelectMany,
    },
    /// Translate selected structure
    Translation {
        /// Selected atoms
        select: SelectMany,
        /// The translation vector
        #[bincode(with_serde)]
        vector: Vector3<f64>,
    },
    TranslationTo {
        select: SelectMany,
        target: SelectOne,
        #[serde(default)]
        #[bincode(with_serde)]
        position: Point3<f64>,
    },
    RotationTo {
        a: SelectOne,
        b: SelectOne,
        select: SelectMany,
        #[serde(default = "Vector3::x")]
        #[bincode(with_serde)]
        direction: Vector3<f64>,
    },
    /// Rotate selected structure
    Rotation {
        /// Selected atoms
        select: SelectMany,
        /// The rotatin center
        #[bincode(with_serde)]
        #[serde(default)]
        center: Point3<f64>,
        /// The axis vector
        #[bincode(with_serde)]
        #[serde(default = "Vector3::x")]
        axis: Vector3<f64>,
        /// The angle of the rotation
        angle: f64,
        /// Use degree instead of radians
        #[serde(default)]
        degree: bool,
    },
    /// Execute an isometry operation on selected structures
    Isometry {
        /// Select atoms
        select: SelectMany,
        /// The isometry operation matrix
        #[bincode(with_serde)]
        isometry: Isometry3<f64>,
    },
    /// Execute Mirror operation on selected structures
    Mirror {
        /// Select atoms
        #[serde(default)]
        select: SelectMany,
        /// The center of the mirror plane
        #[bincode(with_serde)]
        #[serde(default)]
        center: Point3<f64>,
        /// The law vector of the mirror plane
        #[bincode(with_serde)]
        #[serde(default = "Vector3::x")]
        law_vector: Vector3<f64>,
    },
    /// Set atoms to null
    RemoveAtoms {
        select: SelectMany,
    },
    /// Hide a collection of atoms
    Hide {
        select: SelectMany,
    },
    /// Unhide a colletion of atoms
    UnHide {
        select: SelectMany,
    },
}

impl Default for Layer {
    fn default() -> Self {
        Self::Fill {
            data: Default::default(),
        }
    }
}

impl Layer {
    pub fn filter(&self, mut current: SparseMolecule) -> Result<SparseMolecule, LayerStorageError> {
        match self {
            Self::Transparent => {}
            Self::Fill { data } => current.migrate(data.clone()),
            Self::Insert { offset, data } => {
                current.migrate(data.clone().offset(*offset));
            }
            Self::Append { name, data } => {
                let mut molecule = data.clone();
                molecule.ids = molecule.ids.map(|ids| {
                    ids.into_iter()
                        .map(|(private_id, index)| (format!("{}_{}", name, private_id), index))
                        .collect()
                });
                molecule.groups = molecule.groups.map(|groups| {
                    GroupName::from_iter(
                        groups
                            .into_iter()
                            .map(|(group_name, index)| (format!("{}_{}", name, group_name), index)),
                    )
                });
                let molecule = Layer::GroupMap {
                    groups: vec![(name.to_string(), SelectMany::All)],
                }
                .filter(molecule)?;
                let molecule = molecule.offset(current.len());
                current.migrate(molecule);
            }
            Self::SetBond { bonds } => {
                for (a, b, bond) in bonds {
                    let a = a.to_index(&current).ok_or(a.clone())?;
                    let b = b.to_index(&current).ok_or(b.clone())?;
                    current.bonds.set_bond(a, b, Some(*bond));
                }
            }
            Self::SetAtom { atoms } => {
                for (select, atom) in atoms {
                    select.set_atom(&mut current, atom.clone());
                }
            }
            Self::UpdateElement { elements } => {
                for (select, element) in elements {
                    let mut current_atom = select.get_atom(&current).ok_or(select.clone())?;
                    current_atom.element = *element;
                    select.set_atom(&mut current, Some(current_atom));
                }
            }
            Self::UpdateFormalCharge { charges } => {
                for (select, charge) in charges {
                    let mut current_atom = select.get_atom(&current).ok_or(select.clone())?;
                    current_atom.formal_charge = *charge;
                    select.set_atom(&mut current, Some(current_atom));
                }
            }
            Self::AppendAtoms { atoms } => {
                current.atoms.set_atoms(
                    current.atoms.len(),
                    atoms.iter().map(|atom| Some(*atom)).collect(),
                );
            }
            Self::IdMap(data) => {
                let data = data.iter().map(|(name, select)| {
                    Ok((name.to_string(), select.to_index(&current).ok_or(select.clone())?))
                }).collect::<Result<BTreeMap<_, _>, SelectOne>>()?;
                if let Some(current_ids) = &mut current.ids {
                    current_ids.extend(data);
                } else {
                    current.ids = Some(data);
                }
            }
            Self::GroupMap { groups } => {
                for (name, selects) in groups {
                    let selects = selects
                        .to_indexes(&current)
                        .into_iter()
                        .map(|index| (name.to_string(), index));
                    if let Some(current_groups) = &mut current.groups {
                        current_groups.extend(selects);
                    } else {
                        current.groups =
                            Some(GroupName::from_iter(selects.collect::<BTreeSet<_>>()));
                    }
                }
            }
            Self::XYAlign { o, x, y, select } => {
                let o_position = o.get_atom(&current).ok_or(o.clone())?.position;
                let move_to_origin = Point3::origin() - o_position;
                current = Self::Translation {
                    select: select.clone(),
                    vector: move_to_origin,
                }
                .filter(current)?;
                let x_position = x.get_atom(&current).ok_or(x.clone())?.position;
                let ox = (x_position - Point3::origin()).normalize();
                let (ox_rt_axis, ox_rt_angle) = axis_angle_for_b2a(Vector3::x(), ox);
                current = Self::Rotation {
                    select: select.clone(),
                    center: Point3::origin(),
                    axis: *ox_rt_axis,
                    angle: ox_rt_angle,
                    degree: false,
                }
                .filter(current)?;
                let y_position = y.get_atom(&current).ok_or(y.clone())?.position;
                let oy = y_position - Point3::origin();
                let oy = (oy - oy.dot(&Vector3::x()) * Vector3::x()).normalize();
                let (oy_rt_axis, oy_rt_angle) = axis_angle_for_b2a(Vector3::y(), oy);
                current = Self::Rotation {
                    select: select.clone(),
                    center: Default::default(),
                    axis: *oy_rt_axis,
                    angle: oy_rt_angle,
                    degree: false,
                }
                .filter(current)?;
            }
            Self::SetCenter { select, center } => {
                let target_atom = select.get_atom(&current);
                if let Some(target_atom) = target_atom {
                    let translation = center - target_atom.position;
                    let translation =
                        Isometry3::translation(translation.x, translation.y, translation.z);
                    current
                        .atoms
                        .isometry(translation, &SelectMany::All.to_indexes(&current));
                } else {
                    Err(select.clone())?
                }
            }
            Self::DirectionAlign { select, direction } => {
                let target_atom = select.get_atom(&current).ok_or(select.clone())?;
                let current_direction = target_atom.position - Point3::default();
                let (axis, angle) = axis_angle_for_b2a(*direction, current_direction);
                let rotation = Isometry3::rotation(*axis * angle);
                current
                    .atoms
                    .isometry(rotation, &SelectMany::All.to_indexes(&current));
            }
            Self::Translation { select, vector } => {
                let translation = Isometry3::translation(vector.x, vector.y, vector.z);
                current
                    .atoms
                    .isometry(translation, &select.to_indexes(&current));
            }
            Self::TranslationTo {
                select,
                target,
                position,
            } => {
                let target_atom = target.get_atom(&current).ok_or(target.clone())?;
                let vector = *position - target_atom.position;
                current = Self::Translation {
                    select: select.clone(),
                    vector,
                }
                .filter(current)?;
            }
            Self::RotationTo {
                a,
                b,
                select,
                direction,
            } => {
                let center_atom = a.get_atom(&current).ok_or(a.clone())?;
                let target_atom = b.get_atom(&current).ok_or(b.clone())?;
                let current_direction = target_atom.position - center_atom.position;
                let (axis, angle) = axis_angle_for_b2a(*direction, current_direction);
                current = Self::Rotation {
                    select: select.clone(),
                    center: center_atom.position,
                    axis: *axis,
                    angle,
                    degree: false,
                }
                .filter(current)?;
            }
            Self::Rotation {
                select,
                center,
                axis,
                angle,
                degree,
            } => {
                let angle = if *degree { angle * PI / 180. } else { *angle };
                let move_to_origin = Point3::origin() - center;
                let move_to_origin =
                    Translation3::new(move_to_origin.x, move_to_origin.y, move_to_origin.z);
                let move_back = move_to_origin.inverse();
                current
                    .atoms
                    .isometry(move_to_origin.into(), &select.to_indexes(&current));
                current.atoms.isometry(
                    Isometry3::rotation(*axis * angle),
                    &select.to_indexes(&current),
                );
                current
                    .atoms
                    .isometry(move_back.into(), &select.to_indexes(&current));
            }
            Self::Isometry { select, isometry } => {
                current
                    .atoms
                    .isometry(*isometry, &select.to_indexes(&current));
            }
            Self::Mirror {
                select,
                center,
                law_vector,
            } => {
                let selected = select.to_indexes(&current);
                let law_vector = law_vector.normalize();
                let atoms = SparseAtomList::from(
                    current
                        .atoms
                        .data()
                        .iter()
                        .enumerate()
                        .map(|(idx, atom)| {
                            if selected.contains(&idx) {
                                atom.map(|atom| {
                                    let center_atom = atom.position - center;
                                    let projection = center_atom.dot(&law_vector) * law_vector;
                                    let updated_position = atom.position - 2. * projection;
                                    Atom3D {
                                        position: updated_position,
                                        ..atom
                                    }
                                })
                            } else {
                                *atom
                            }
                        })
                        .collect::<Vec<_>>(),
                );
                current.atoms.migrate(atoms);
            }
            Self::RemoveAtoms { select } => {
                let selected = select.to_indexes(&current);
                let atoms = SparseAtomList::from(
                    current.atoms.data().iter().enumerate()
                        .map(|(index, _)| {
                            if selected.contains(&index) {
                                Some(Atom3D::default())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>(),
                );
                current.atoms.migrate(atoms);
            }
            Self::Hide { select } => {
                let selected = select.to_indexes(&current);
                let atoms = current.atoms.data().iter().enumerate().map(|(idx, atom)| {
                    if selected.contains(&idx) {
                        Ok(if let Some(atom) = atom {
                            Some(Atom3D {
                                element: atom.element.checked_add(128).ok_or((idx, atom.element))?,
                                ..*atom
                            })
                        } else {
                            None
                        })
                    } else {
                        Ok(None)
                    }
                }).collect::<Result<Vec<_>,LayerStorageError>>()?;
                current.atoms.migrate(SparseAtomList::from(atoms));
            }
            Self::UnHide { select } => {
                let selected = select.to_indexes(&current);
                let atoms = current
                    .atoms
                    .data()
                    .iter()
                    .enumerate()
                    .map(|(idx, atom)| {
                        if selected.contains(&idx) {
                            Ok(if let Some(atom) = atom {
                                Some(Atom3D {
                                    element: atom
                                        .element
                                        .checked_sub(128)
                                        .ok_or((idx, atom.element))?,
                                    ..*atom
                                })
                            } else {
                                None
                            })
                        } else {
                            Ok(None)
                        }
                    })
                    .collect::<Result<Vec<_>, LayerStorageError>>()?;

                current.atoms.migrate(SparseAtomList::from(atoms));
            }
        }
        Ok(current)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, PartialOrd, Ord, Eq, Encode, Decode)]
#[serde(untagged)]
pub enum SelectOne {
    Index(usize),
    IdName(String),
}

impl Display for SelectOne {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl std::error::Error for SelectOne {
    fn description(&self) -> &str {
        "Unable to find the atom specified by the element"
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl SelectOne {
    pub fn to_index(&self, layer: &SparseMolecule) -> Option<usize> {
        match self {
            Self::Index(index) => Some(*index),
            Self::IdName(id_name) => layer.ids.as_ref()?.get(id_name).copied(),
        }
    }

    pub fn get_atom(&self, layer: &SparseMolecule) -> Option<Atom3D> {
        self.to_index(layer)
            .and_then(|index| layer.atoms.read_atom(index))
    }

    pub fn set_atom(&self, layer: &mut SparseMolecule, atom: Option<Atom3D>) -> Option<()> {
        self.to_index(layer)
            .and_then(|index| Some(layer.atoms.set_atoms(index, vec![atom])))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode, Default)]
#[serde(untagged)]
pub enum SelectMany {
    #[default]
    All,
    Complex {
        includes: Vec<SelectMany>,
        #[serde(default)]
        excludes: Vec<SelectMany>,
    },
    Element(usize),
    Indexes(BTreeSet<SelectOne>),
    Range(RangeInclusive<usize>),
    GroupName(String),
}

impl SelectMany {
    pub fn to_indexes(&self, layer: &SparseMolecule) -> BTreeSet<usize> {
        match self {
            Self::All => (0..layer.atoms.len()).collect(),
            Self::Element(number) => (0..layer.atoms.len())
                .filter(|index| {
                    if let Some(atom) = layer.atoms.read_atom(*index) {
                        atom.element == *number
                    } else {
                        false
                    }
                })
                .collect(),
            Self::GroupName(group_name) => layer
                .groups
                .as_ref()
                .map(|groups| groups.get_left(group_name).into_iter().copied().collect())
                .unwrap_or_default(),
            Self::Indexes(indexes) => indexes
                .iter()
                .filter_map(|select| select.to_index(layer))
                .collect(),
            Self::Range(range) => range.clone().collect(),
            Self::Complex { includes, excludes } => {
                let mut selected = BTreeSet::new();
                for include in includes {
                    selected.extend(include.to_indexes(layer));
                }
                for exclude in excludes {
                    let exclude = exclude.to_indexes(layer);
                    selected.retain(|index| !exclude.contains(index));
                }
                selected
            }
        }
    }
}

impl Value for Layer {
    type AsBytes<'a> = Vec<u8>;
    type SelfType<'a> = Layer;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::decode_from_slice(data, bincode::config::standard())
            .unwrap()
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        bincode::encode_to_vec(value, bincode::config::standard()).unwrap()
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("layer_table")
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum LayerStorageError {
    NoSuchLayer(u64),
    SelectNotFound(SelectOne),
    HideOverflow { idx: usize, current_value: usize },
}

impl From<SelectOne> for LayerStorageError {
    fn from(value: SelectOne) -> Self {
        Self::SelectNotFound(value)
    }
}

impl From<(usize, usize)> for LayerStorageError {
    fn from(value: (usize, usize)) -> Self {
        Self::HideOverflow {
            idx: value.0,
            current_value: value.1,
        }
    }
}

impl std::fmt::Display for LayerStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for LayerStorageError {}
