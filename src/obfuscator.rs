use crate::ast::RegexNode;
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
                RegexNode::CharacterClass {
                    negated,
                    chars: chars.into_iter()
                        .map(|c| c)  // Will be replaced with Unicode escapes in string conversion
                        .collect(),
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
            chars: vec![c],  // Will be converted to Unicode escape in string conversion
        }
    }
} 