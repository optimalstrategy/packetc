pub struct ImplCtx<'a> {
    pub indentation: String,
    pub out: &'a mut String,
    pub stack: Vec<String>,
}

impl<'a> ImplCtx<'a> {
    pub fn new(out: &'a mut String) -> ImplCtx {
        ImplCtx {
            indentation: String::new(),
            out,
            stack: Vec::new(),
        }
    }

    pub fn push_indent(&mut self) {
        self.indentation += "    ";
    }

    pub fn pop_indent(&mut self) {
        self.indentation.truncate(if self.indentation.len() < 4 {
            0
        } else {
            self.indentation.len() - 4
        });
    }

    pub fn push_fname<S: Into<String>>(&mut self, name: S) {
        self.stack.push(name.into());
    }

    pub fn pop_fname(&mut self) {
        self.stack.pop();
    }

    pub fn swap_stack(&mut self, other: &mut Vec<String>) {
        std::mem::swap(&mut self.stack, other);
    }
}
