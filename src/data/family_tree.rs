use std::collections::HashMap;
use serde::Deserialize;
use super::person::Person;

/// YAML input format for a family
#[derive(Debug, Deserialize)]
pub struct FamilyInput {
    pub family: FamilyMeta,
    pub people: Vec<Person>,
}

#[derive(Debug, Deserialize)]
pub struct FamilyMeta {
    pub name: String,
    pub root: String,
}

/// Parsed and indexed family tree
#[derive(Debug, Clone)]
pub struct FamilyTree {
    pub name: String,
    pub root_id: String,
    pub people: HashMap<String, Person>,
}

impl FamilyTree {
    /// Parse from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self, String> {
        let input: FamilyInput = serde_yaml::from_str(yaml)
            .map_err(|e| format!("YAML parse error: {}", e))?;

        let mut people = HashMap::new();
        for person in input.people {
            people.insert(person.id.clone(), person);
        }

        // Validate root exists
        if !people.contains_key(&input.family.root) {
            return Err(format!("Root person '{}' not found in people list", input.family.root));
        }

        // Validate all children references exist
        for person in people.values() {
            for child_id in &person.children {
                if !people.contains_key(child_id) {
                    return Err(format!(
                        "Child '{}' referenced by '{}' not found",
                        child_id, person.id
                    ));
                }
            }
        }

        Ok(Self {
            name: input.family.name,
            root_id: input.family.root,
            people,
        })
    }

    /// Get the root person
    pub fn root(&self) -> Option<&Person> {
        self.people.get(&self.root_id)
    }

    /// Get a person by ID
    pub fn get(&self, id: &str) -> Option<&Person> {
        self.people.get(id)
    }

    /// Get children of a person
    pub fn children_of(&self, id: &str) -> Vec<&Person> {
        self.people
            .get(id)
            .map(|p| {
                p.children
                    .iter()
                    .filter_map(|cid| self.people.get(cid))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Count total people
    pub fn len(&self) -> usize {
        self.people.len()
    }

    pub fn is_empty(&self) -> bool {
        self.people.is_empty()
    }

    /// Calculate max depth of tree
    pub fn max_depth(&self) -> usize {
        fn depth_from(tree: &FamilyTree, id: &str) -> usize {
            let children = tree.children_of(id);
            if children.is_empty() {
                1
            } else {
                1 + children
                    .iter()
                    .map(|c| depth_from(tree, &c.id))
                    .max()
                    .unwrap_or(0)
            }
        }
        depth_from(self, &self.root_id)
    }

    /// Iterate over all people in pre-order (root first)
    pub fn iter_preorder(&self) -> PreorderIter<'_> {
        PreorderIter {
            tree: self,
            stack: vec![self.root_id.clone()],
        }
    }
}

pub struct PreorderIter<'a> {
    tree: &'a FamilyTree,
    stack: Vec<String>,
}

impl<'a> Iterator for PreorderIter<'a> {
    type Item = &'a Person;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.stack.pop()?;
        let person = self.tree.people.get(&id)?;

        // Push children in reverse order so first child is processed first
        for child_id in person.children.iter().rev() {
            self.stack.push(child_id.clone());
        }

        Some(person)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_YAML: &str = r#"
family:
  name: "Test Family"
  root: "grandparent"

people:
  - id: "grandparent"
    name: "Grand Parent"
    biography: "The founder of our family line."
    birth_year: 1920
    death_year: 2000
    children:
      - "parent1"
      - "parent2"

  - id: "parent1"
    name: "Parent One"
    biography: "First child."
    children:
      - "child1"

  - id: "parent2"
    name: "Parent Two"
    biography: "Second child."
    children: []

  - id: "child1"
    name: "Child One"
    biography: "The youngest generation."
"#;

    #[test]
    fn test_parse_yaml() {
        let tree = FamilyTree::from_yaml(SAMPLE_YAML).unwrap();
        assert_eq!(tree.name, "Test Family");
        assert_eq!(tree.len(), 4);
    }

    #[test]
    fn test_root_access() {
        let tree = FamilyTree::from_yaml(SAMPLE_YAML).unwrap();
        let root = tree.root().unwrap();
        assert_eq!(root.name, "Grand Parent");
    }

    #[test]
    fn test_children_access() {
        let tree = FamilyTree::from_yaml(SAMPLE_YAML).unwrap();
        let children = tree.children_of("grandparent");
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_max_depth() {
        let tree = FamilyTree::from_yaml(SAMPLE_YAML).unwrap();
        assert_eq!(tree.max_depth(), 3); // grandparent -> parent1 -> child1
    }

    #[test]
    fn test_preorder_iteration() {
        let tree = FamilyTree::from_yaml(SAMPLE_YAML).unwrap();
        let names: Vec<_> = tree.iter_preorder().map(|p| p.name.as_str()).collect();
        assert_eq!(names[0], "Grand Parent");
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn test_invalid_root() {
        let yaml = r#"
family:
  name: "Bad"
  root: "nonexistent"
people:
  - id: "someone"
    name: "Someone"
"#;
        let result = FamilyTree::from_yaml(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_invalid_child_reference() {
        let yaml = r#"
family:
  name: "Bad"
  root: "parent"
people:
  - id: "parent"
    name: "Parent"
    children:
      - "missing-child"
"#;
        let result = FamilyTree::from_yaml(yaml);
        assert!(result.is_err());
    }
}
