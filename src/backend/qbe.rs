use crate::backend::{Backend, BackendOutput};
use crate::parser::{
    Info,
    ast::{AssignOp, Stmt},
    expr::{ArithOp, CompOp, Expr, LogicOp, UnaryOp},
    types::Type,
};
use crate::report::{Diagnostic, Level, Phase};
use crate::session::{Session, source::FileId};
use crate::{diag, span_type};
use qbe::{Block, Cmp, DataDef, DataItem, Function, Instr, Linkage, Module, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

macro_rules! emit_error_type {
    ($self:expr, $expr:expr,$notes:expr, $helps:expr, $($msg:tt)*) => {
        $self.emit_error(diag!(
            Level::Error,
            span_type!($self.file_id, $expr),
            Phase::CodeGen,
            $notes,
            $helps,
            $($msg)*
         ));
    };
}

pub struct QbeBackend<'a> {
    session: Rc<RefCell<Session<'a>>>,
    file_id: FileId,
    module: Module,

    label_count: usize,
    tmp_count: usize,
    lit_count: usize,

    scopes: Vec<HashMap<String, Type>>,
    /// last loop start and exit labels
    loop_stack: Vec<(String, String)>,
    /// Functions id, return type and types of arguments
    functions: HashMap<String, (Option<Type>, Vec<Type>)>,
    /// String data (str -> literal)
    strings: HashMap<String, String>,
    /// Uses list for build right function names
    uses: Vec<Vec<String>>,

    current_func: String,
}

impl<'a> QbeBackend<'a> {
    pub fn new(session: Rc<RefCell<Session<'a>>>) -> Self {
        Self {
            session,
            module: Module::new(),
            strings: HashMap::new(),
            functions: HashMap::new(),
            current_func: String::new(),
            label_count: 0,
            tmp_count: 0,
            lit_count: 0,
            file_id: 0,
            loop_stack: vec![],
            scopes: vec![],
            uses: vec![],
        }
    }
}

impl Backend for QbeBackend<'_> {
    fn compile(&mut self, file_id: FileId, ast: &[Stmt]) -> BackendOutput {
        self.file_id = file_id;
        self.functions.clear();
        self.scopes.clear();
        for stmt in ast {
            self.gen_stmt(stmt);
        }
        BackendOutput::Text(self.module.to_string())
    }

    fn ext(&self) -> String {
        String::from(".ssa")
    }

    fn out_dir(&self) -> String {
        String::from("/qbe")
    }
}

impl QbeBackend<'_> {
    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Func(is_exp, _is_pub, id, args, ret_ty, body) => {
                self.current_func = id.to_string();
                let linkable = if id == "main" || *is_exp {
                    Linkage::public()
                } else {
                    Linkage::private()
                };
                self.functions.insert(
                    id.clone(),
                    (
                        ret_ty.clone(),
                        args.iter().map(|(_id, ty)| ty.clone()).collect(),
                    ),
                );
                let ret_ty = if let Some(ty) = ret_ty {
                    Some(ty.to_qbe(self))
                } else {
                    None
                };
                let mut func =
                    Function::new(linkable, id, self.to_qbe_func_args(args.to_vec()), ret_ty);
                func.add_block("start");
                self.push_scope();
                for body_stmt in body {
                    self.gen_body_stmt(&mut func, &body_stmt);
                }
                self.pop_scope();
                self.module.add_function(func.clone());
            }
            Stmt::Use(_is_pub, pathes) => {
                for path in pathes {
                    self.uses.push(path.to_vec());
                }
            }
            _ => {
                unreachable!("Other statements should be in function body!")
            }
        }
    }

    fn gen_body_stmt(&mut self, func: &mut Function, stmt: &Stmt) {
        match stmt {
            Stmt::Declare(_mut_kind, id, ty, val) => {
                let instr_alloc = match ty.size(self) {
                    (4, n) => Instr::Alloc4(n as u32),
                    (8, n) => Instr::Alloc8(n as u64),
                    (16, n) => Instr::Alloc16(n),
                    _ => unreachable!(),
                };
                let block = get_block(func);
                block.assign_instr(Value::Temporary(id.clone()), ty.to_qbe(self), instr_alloc);
                self.push_var(id.clone(), ty.clone());
                self.gen_body_stmt(
                    func,
                    &Stmt::Assign(false, id.clone(), AssignOp::default(), val.clone()),
                );
            }
            Stmt::Assign(_is_deref, id, _assign_op, val) => {
                let ty = self.var_info(id.clone());
                let block = func.blocks.last_mut().unwrap();
                let value = self.gen_expr(val.clone(), block, ty.clone());
                block.assign_instr(
                    Value::Temporary(id.clone()),
                    ty.to_qbe(self),
                    Instr::Copy(value.1),
                );
            }
            Stmt::Expr(expr) => {
                let block = func.blocks.last_mut().unwrap();
                self.gen_expr(expr.clone(), block, Type::Unknown);
            }
            Stmt::While(cond, body) => {
                let check_label = self.new_label("while_check");
                let start_label = self.new_label("while_start");
                let exit_label = self.new_label("while_end");

                self.loop_stack
                    .push((start_label.clone(), exit_label.clone()));

                let check_block = func.add_block(&check_label);
                let value = self
                    .gen_expr(cond.clone(), check_block, Type::Bool(Info::empty()))
                    .1;
                check_block.add_instr(Instr::Jnz(value, start_label.clone(), exit_label.clone()));
                func.add_block(&start_label);
                for body_stmt in body {
                    self.gen_body_stmt(func, body_stmt);
                }
                get_block(func).add_instr(Instr::Jmp(check_label.clone()));
                func.add_block(&exit_label);

                self.loop_stack.pop();
            }
            Stmt::IfElse(branches) => {
                let exit_label = self.new_label("if_exit");
                for (i, (cond, body)) in branches.iter().enumerate() {
                    let is_last = i == branches.len() - 1;
                    let check_label = self.new_label("if_check");
                    let start_label = self.new_label("if_start");

                    let cond = if let Some(some_cond) = cond {
                        some_cond
                    } else {
                        // handle else block (all elifs handle after let cond)
                        func.add_block(start_label);
                        for body_stmt in body {
                            self.gen_body_stmt(func, body_stmt);
                        }
                        break;
                    };
                    // Build check block
                    let check_block = func.add_block(check_label);
                    let cond = self
                        .gen_expr(cond.clone(), check_block, Type::Bool(cond.info()))
                        .1;
                    let exit = if !is_last {
                        format!("check_{}", self.label_count)
                    } else {
                        exit_label.clone()
                    };
                    check_block.add_instr(Instr::Jnz(cond, start_label.clone(), exit));
                    // Build start block
                    func.add_block(start_label);
                    for body_stmt in body {
                        self.gen_body_stmt(func, body_stmt);
                    }
                    get_block(func).add_instr(Instr::Jmp(exit_label.clone()));
                }
                func.add_block(exit_label);
            }
            Stmt::Return(value) => {
                let result = match value {
                    None => None,
                    Some(expr) => {
                        let ret = self
                            .gen_expr(
                                expr.clone(),
                                get_block(func),
                                self.functions
                                    .get(&self.current_func)
                                    .unwrap()
                                    .clone()
                                    .0
                                    .unwrap(),
                            )
                            .1;
                        Some(ret)
                    }
                };
                get_block(func).add_instr(Instr::Ret(result));
            }
            Stmt::Break => {
                let goto = self.loop_stack.last().unwrap().1.clone();
                get_block(func).add_instr(Instr::Jmp(goto));
            }
            Stmt::Continue => {
                let goto = self.loop_stack.last().unwrap().0.clone();
                get_block(func).add_instr(Instr::Jmp(goto));
            }
            _ => {
                panic!("Other statements cannot be in function body!")
            }
        }
    }

    fn gen_expr(&mut self, expr: Expr, block: &mut Block, ty: Type) -> (String, Value, Type) {
        let tmp = self.new_tmp();
        let value = match expr.clone() {
            Expr::Int(n, info) => {
                let value = Value::Const(n as u64);
                (value, Type::I32(info))
            }
            Expr::Float(n, info) => {
                let value = Value::Const(n.to_bits());
                (value, Type::F32(info))
            }
            Expr::Id(id, _info) => {
                let id = id.join("_");
                let value = Value::Temporary(id.clone());
                (value, self.var_info(id))
            }
            Expr::Bool(truth, info) => {
                let truth = if truth { 1 } else { 0 };
                let value = Value::Const(truth);
                (value, Type::Bool(info))
            }
            Expr::Str(s, info) => {
                let value = Value::Global(self.new_string(&s));
                (value, Type::Str(info))
            }
            Expr::Arith(left, op, right, _info) => {
                let (_left_tmp, left_value, left_ty) = self.gen_expr(*left, block, ty.clone());
                let (_right_tmp, right_value, _right_ty) = self.gen_expr(*right, block, ty.clone());
                let instr = match op {
                    ArithOp::Add => Instr::Add(left_value, right_value),
                    ArithOp::Div => Instr::Div(left_value, right_value),
                    ArithOp::Mul => Instr::Mul(left_value, right_value),
                    ArithOp::Sub => Instr::Sub(left_value, right_value),
                };
                let value = Value::Temporary(tmp.clone());
                block.assign_instr(value.clone(), ty.to_qbe(self), instr);
                (value, left_ty)
            }
            Expr::Logic(left, op, right, _info) => {
                let (_left_tmp, left_value, left_ty) = self.gen_expr(*left, block, ty.clone());
                let (_right_tmp, right_value, _right_ty) = self.gen_expr(*right, block, ty.clone());
                let instr = match op {
                    LogicOp::And => Instr::And(left_value, right_value),
                    LogicOp::Or => Instr::Or(left_value, right_value),
                };
                let value = Value::Temporary(tmp.clone());
                block.assign_instr(value.clone(), ty.to_qbe(self), instr);
                (value, left_ty)
            }
            Expr::Comp(left, op, right, info) => {
                let (_left_tmp, left_value, left_ty) = self.gen_expr(*left, block, ty.clone());
                let (_right_tmp, right_value, _right_ty) = self.gen_expr(*right, block, ty.clone());
                let cmp = match op {
                    CompOp::Eq => Cmp::Eq,
                    CompOp::Ne => Cmp::Eq,
                    CompOp::Gt => {
                        if left_ty.is_unsigned() {
                            Cmp::Ugt
                        } else {
                            Cmp::Sgt
                        }
                    }
                    CompOp::Ge => {
                        if left_ty.is_unsigned() {
                            Cmp::Uge
                        } else {
                            Cmp::Sge
                        }
                    }
                    CompOp::Lt => {
                        if left_ty.is_unsigned() {
                            Cmp::Ult
                        } else {
                            Cmp::Slt
                        }
                    }
                    CompOp::Le => {
                        if left_ty.is_unsigned() {
                            Cmp::Ule
                        } else {
                            Cmp::Sle
                        }
                    }
                };
                let instr = Instr::Cmp(qbe::Type::Word, cmp, left_value, right_value);
                let value = Value::Temporary(tmp.clone());
                block.assign_instr(value.clone(), ty.to_qbe(self), instr);
                (value, Type::Bool(info))
            }
            Expr::Unary(op, expr, _info) => {
                let (_left_tmp, value, ty) = self.gen_expr(*expr.clone(), block, ty.clone());
                let instr = match op {
                    UnaryOp::Neg => Instr::Neg(value),
                    UnaryOp::Not => {
                        let (not_tmp, _not_value, _not_ty) = self.gen_expr(
                            Expr::Comp(
                                expr.clone(),
                                CompOp::Ne,
                                Box::new(Expr::Int(0, Info::empty())),
                                Info::empty(),
                            ),
                            block,
                            ty.clone(),
                        );
                        Instr::Copy(Value::Temporary(not_tmp))
                    }
                };
                let value = Value::Temporary(tmp.clone());
                block.assign_instr(value.clone(), ty.to_qbe(self), instr);
                (value, ty)
            }
            Expr::Call(id, args, _info) => {
                let func_id = match *id {
                    Expr::Id(path, _) => self.build_call_id(path),
                    _ => unreachable!(),
                };
                let args = self.to_qbe_args(block, func_id.clone(), args);
                block.add_instr(Instr::Call(func_id.clone(), args, None));
                (
                    Value::Temporary(tmp.clone()),
                    self.functions
                        .get(&func_id)
                        .unwrap()
                        .0
                        .clone()
                        .unwrap_or(Type::Unknown),
                )
            }
            Expr::Invalid => {
                unreachable!("Invalid values cannot be in backend")
            }
        };
        (tmp, value.0, value.1)
    }

    fn emit_error(&mut self, diag: Diagnostic) {
        self.session.borrow_mut().emit_error(diag);
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}_{}", self.label_count);
        self.label_count += 1;
        label
    }

    fn new_tmp(&mut self) -> String {
        let tmp = format!("_tmp_{}", self.tmp_count);
        self.tmp_count += 1;
        tmp
    }

    fn new_string(&mut self, s: &str) -> String {
        if self.strings.contains_key(s) {
            return self.strings.get(s).unwrap().to_string();
        }
        let lit = format!("_lit_{}", self.lit_count);
        self.strings.insert(s.to_string(), lit.clone());
        let item = (qbe::Type::Byte, DataItem::Str(s.to_string()));
        let zero = (qbe::Type::Byte, DataItem::Const(0));
        let items = vec![item, zero];
        self.module
            .add_data(DataDef::new(Linkage::private(), &lit, None, items));
        self.lit_count += 1;
        lit
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn var_info(&self, id: String) -> Type {
        for scope in &self.scopes {
            for (var_id, ty) in scope {
                if *var_id == id {
                    return ty.clone();
                }
            }
        }
        Type::Unknown
    }

    fn push_var(&mut self, id: String, ty: Type) {
        for scope in &mut self.scopes {
            for var_id in scope.keys() {
                if *var_id == id {
                    scope.insert(id, ty);
                    return;
                }
            }
        }
        if let Some(last) = self.scopes.last_mut() {
            last.insert(id, ty);
        }
    }

    fn to_qbe_func_args(&mut self, args: Vec<(String, Type)>) -> Vec<(qbe::Type, Value)> {
        let mut new_args = vec![];
        for (id, ty) in args {
            let val = Value::Temporary(id);
            let ty = ty.to_qbe(self);
            new_args.push((ty, val));
        }
        new_args
    }

    fn to_qbe_args(
        &mut self,
        block: &mut Block,
        id: String,
        args: Vec<Expr>,
    ) -> Vec<(qbe::Type, Value)> {
        let mut new_args = vec![];
        for (i, expr) in args.iter().enumerate() {
            let ty = self.functions.get(&id).expect(&id).1[i].clone();
            let value = self.gen_expr(expr.clone(), block, ty.clone());
            new_args.push((ty.to_qbe(self), value.1));
        }
        new_args
    }

    fn build_call_id(&self, path: Vec<String>) -> String {
        for use_loc in &self.uses {
            if use_loc.last() == path.first() {
                let full = use_loc
                    .join("_")
                    .push_str(&("_".to_owned() + &path[1..].join("_")));
                full
            }
        }
        path.join("_")
    }
}

/// Get current block from QBE function
fn get_block(func: &mut Function) -> &mut Block {
    func.blocks.last_mut().unwrap()
}

impl Type {
    /// Get alloc variant type and size: (alloc4 1)
    pub fn size(&self, backend: &mut QbeBackend) -> (u8, u128) {
        match self {
            Self::I32(_) | Self::U32(_) | Self::F32(_) | Self::Bool(_) => (4, 1),
            Self::Ptr(..) | Self::Str(_) | Self::Array(..) | Self::Custom(..) => (8, 1),

            Self::Unknown => {
                emit_error_type!(
                    backend,
                    self,
                    vec!["Use correct type"],
                    vec![],
                    "Invalid type: {}",
                    self
                );
                (4, 1)
            }
        }
    }
    /// Convert Krez to QBE type
    pub fn to_qbe(&self, backend: &mut QbeBackend) -> qbe::Type {
        match self {
            Self::I32(_) | Self::U32(_) | Self::Bool(_) => qbe::Type::Word,
            Self::Ptr(..) | Self::Str(_) | Self::Array(..) | Self::Custom(..) => qbe::Type::Long,
            Self::F32(_) => qbe::Type::Single,
            Self::Unknown => {
                emit_error_type!(
                    backend,
                    self,
                    vec!["Use correct type"],
                    vec![],
                    "Invalid type: {}",
                    self
                );
                qbe::Type::Zero
            }
        }
    }
}
