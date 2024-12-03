use crate::ast::{RegexNode, GroupKind};
use rand::thread_rng;

pub struct Obfuscator {
    rng: rand::rngs::ThreadRng,
}

impl Obfuscator {
    pub fn new() -> Self {
        Obfuscator {
            rng: thread_rng(),
        }
    }

    pub fn obfuscate(&mut self, ast: Vec<RegexNode>) -> Vec<RegexNode> {
        ast.into_iter()
            .map(|node| self.obfuscate_node(node))
            .collect()
    }

    fn obfuscate_node(&mut self, node: RegexNode) -> RegexNode {
        match node {
            RegexNode::Literal(c) => self.obfuscate_literal(c),
            RegexNode::CharacterClass { negated, chars } => {
                if negated {
                    // Keep negated character classes as is for now
                    RegexNode::CharacterClass { negated, chars }
                } else {
                    // Convert character class to alternation of single-char classes
                    let alternatives: Vec<Vec<RegexNode>> = chars.into_iter()
                        .map(|c| {
                            vec![RegexNode::CharacterClass {
                                negated: false,
                                chars: vec![c],
                            }]
                        })
                        .collect();

                    // Wrap in a non-capturing group
                    RegexNode::Group(
                        GroupKind::NonCapturing,
                        vec![RegexNode::Alternation(alternatives)]
                    )
                }
            }
            RegexNode::Quantified { node, quantifier } => RegexNode::Quantified {
                node: Box::new(self.obfuscate_node(*node)),
                quantifier,
            },
            RegexNode::Group(kind, nodes) => RegexNode::Group(
                kind,
                nodes.into_iter()
                    .map(|node| self.obfuscate_node(node))
                    .collect(),
            ),
            RegexNode::Alternation(alternatives) => RegexNode::Alternation(
                alternatives
                    .into_iter()
                    .map(|alt| {
                        alt.into_iter()
                            .map(|node| self.obfuscate_node(node))
                            .collect()
                    })
                    .collect(),
            ),
            // For other node types, return as is
            _ => node,
        }
    }

    fn obfuscate_literal(&mut self, c: char) -> RegexNode {
        RegexNode::CharacterClass {
            negated: false,
            chars: vec![c],
        }
    }
} 