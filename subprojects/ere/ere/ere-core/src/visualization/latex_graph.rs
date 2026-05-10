use crate::visualization::layout::{BuildLayout, DAGLayout};

pub fn escape_latex(text: impl AsRef<str>) -> String {
    return text
        .as_ref()
        .chars()
        .map(|c| match c {
            '\\' => r"{\textbackslash}".to_string(),
            '&' => r"\&".to_string(),
            '%' => r"\%".to_string(),
            '$' => r"\$".to_string(),
            '#' => r"\#".to_string(),
            '_' => r"\_".to_string(),
            '{' => r"\{".to_string(),
            '}' => r"\}".to_string(),
            '~' => r"{\textasciitilde}".to_string(),
            '^' => r"{\textasciicircum}".to_string(),
            c => c.to_string(),
        })
        .collect();
}

pub struct LatexGraphTransition {
    pub(crate) to: usize,
    /// The label is a valid latex-encoded string to be inserted at the label.
    pub(crate) label: String,
}
impl LatexGraphTransition {
    pub fn display_in_line(&self, from: usize) -> String {
        let label = &self.label;
        let bend = match self.to.cmp(&from) {
            std::cmp::Ordering::Less => "[bend left] ",
            std::cmp::Ordering::Equal => "[loop below]",
            std::cmp::Ordering::Greater => "[bend left] ",
        };
        format!(
            "\\path[->] (q{from}) edge {bend} node {{{label}}} (q{});\n",
            self.to
        )
    }
    pub fn display_straight(&self, from: usize) -> String {
        let label = &self.label;
        format!(
            "\\path[->] (q{from}) edge node {{{label}}} (q{});\n",
            self.to
        )
    }
}

pub struct LatexGraphState {
    /// The label is a valid latex-encoded string to be inserted at the label.
    pub(crate) label: String,
    pub(crate) transitions: Vec<LatexGraphTransition>,
    pub(crate) initial: bool,
    pub(crate) accept: bool,
}
impl LatexGraphState {
    /// All states are just in a horizontal line
    pub fn display_in_line(&self, idx: usize) -> String {
        let mut modifiers = String::new();
        if self.initial {
            modifiers += ", initial";
        }
        if self.accept {
            modifiers += ", accepting"
        }
        let label = escape_latex(&self.label);
        if idx == 0 {
            return format!("\\node[state{modifiers}](q0){{{label}}};\n",);
        } else {
            return format!(
                "\\node[state{modifiers}, right of=q{}](q{idx}){{{label}}};\n",
                idx - 1,
            );
        }
    }

    pub fn display_at(&self, idx: usize, x: f64, y: f64) -> String {
        let mut modifiers = String::new();
        if self.initial {
            modifiers += ", initial";
        }
        if self.accept {
            modifiers += ", accepting"
        }
        let label = escape_latex(&self.label);
        return format!("\\node[state{modifiers}](q{idx}) at ({x}, {y}) {{{label}}};\n",);
    }
}

/// Used for tikz visualizations of NFA-like graphs
pub struct LatexGraph {
    pub(crate) states: Vec<LatexGraphState>,
}
impl LatexGraph {
    /// Writes a LaTeX TikZ representation to visualize the graph.
    ///
    /// If `include_doc` is `true`, will include the headers.
    /// Otherwise, you should include `\usepackage{tikz}` and `\usetikzlibrary{automata, positioning}`.
    pub fn to_tikz(&self, include_doc: bool) -> String {
        let layout = self.pick_layout().map(|layout| layout.layout());

        let mut text_parts: Vec<String> = Vec::new();
        if include_doc {
            text_parts.push(
                "\\documentclass{standalone}\n\\usepackage{tikz}\n\\usetikzlibrary{automata, positioning}\n\\begin{document}\n"
                .into(),
            );
        }
        text_parts.push("\\begin{tikzpicture}[node distance=2cm, auto]\n".into());

        let mut transition_parts = Vec::new();

        for (i, state) in self.states.iter().enumerate() {
            if let Some(layout) = layout.as_ref() {
                let (x, y) = layout[i];
                text_parts.push(state.display_at(i, x * 2.0, y * 2.0));
            } else {
                text_parts.push(state.display_in_line(i));
            }

            for tr in &state.transitions {
                if let Some(layout) = layout.as_ref() {
                    let x = layout[i].0;
                    let x_next = layout[tr.to].0;
                    if x_next - x > 1.0 {
                        transition_parts.push(tr.display_in_line(i));
                    } else {
                        transition_parts.push(tr.display_straight(i));
                    }
                } else {
                    transition_parts.push(tr.display_in_line(i));
                }
            }
        }
        text_parts.extend_from_slice(&transition_parts);

        text_parts.push("\\end{tikzpicture}\n".into());
        if include_doc {
            text_parts.push("\\end{document}\n".into());
        }
        return text_parts.into_iter().collect();
    }

    fn pick_layout<'a>(&'a self) -> Option<Box<dyn BuildLayout + 'a>> {
        if let Some(dag) = DAGLayout::new(self) {
            return Some(Box::new(dag));
        }

        // TODO: more layouts
        return None;
    }
}
