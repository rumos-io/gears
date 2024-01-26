use std::cmp::Ordering;

use super::node_v2::{NodeEnum, NodeTrait};

#[derive(Debug, Default)]
pub struct AvlTree {
    pub(crate) root: Option<Box<NodeEnum>>,
}

impl AvlTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_node(root: NodeEnum) -> Self {
        Self {
            root: Some(Box::new(root)),
        }
    }

    pub fn get(&self, key: impl AsRef<[u8]>) -> Result< Option<&NodeEnum>, LeafNodeError > {
        // Option<&Box<T>> -> Option<&T>
        let mut current_tree : Option< &NodeEnum> = match &self.root 
        {
            Some( var ) => Some( &*var ),
            None => None,
        };

        while let Some(current_node) = current_tree {

            match current_node
            {
                NodeEnum::Inner( inner ) => 
                {
                    match inner.key().cmp(key.as_ref()) {
                        Ordering::Less => {
                            current_tree = current_node.right_node();
                        }
                        Ordering::Equal => {
                            return Ok( Some(&current_node) );
                        }
                        Ordering::Greater => {
                            current_tree = current_node.left_node();
                        }
                    };
                },
                NodeEnum::Leaf( leaf ) => 
                {
                    return match leaf.key() == key.as_ref()
                    {
                        true => Ok( Some( current_node) ),
                        false =>  
                        {
                            let mut err_key = Vec::new();

                            err_key.extend_from_slice( key.as_ref() );

                            Err( LeafNodeError( err_key ) )
                        },
                    }
                },
            }
        }

        Ok( None )
    }
}

#[ derive(Debug, Clone, thiserror::Error)]
#[error("There is no value after leaf node. Not found key: {0:?}")]
pub struct LeafNodeError( pub Vec<u8>);