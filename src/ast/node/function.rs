use super::statement::StatementsNode;
use super::types::*;
use super::*;
use super::{alloc, types::TypedIdentifierNode, Node};
use crate::ast::ctx::{FNType, PLType};
use crate::ast::node::{deal_line, tab};
use inkwell::debug_info::*;
use inkwell::values::FunctionValue;
use internal_macro::range;
use std::fmt::format;

use lazy_static::__Deref;

#[range]
pub struct FuncDefNode {
    pub typenode: FuncTypeNode,
    pub body: Option<StatementsNode>,
}

impl FuncDefNode {
    pub fn print(&self, tabs: usize, end: bool, mut line: Vec<bool>) {
        deal_line(tabs, &mut line, end);
        tab(tabs, line.clone(), end);
        println!("FuncDefNode");
        tab(tabs + 1, line.clone(), end);
        println!("id: {}", self.typenode.id);
        for p in self.typenode.paralist.iter() {
            p.print(tabs + 1, false, line.clone());
        }
        if let Some(body) = &self.body {
            tab(tabs + 1, line.clone(), false);
            println!("type: {}", self.typenode.ret.id);
            body.print(tabs + 1, true, line.clone());
        } else {
            tab(tabs + 1, line, true);
            println!("type: {}", self.typenode.ret.id);
        }
    }
    pub fn emit_func_def<'a, 'ctx>(
        &'a mut self,
        ctx: &mut crate::ast::ctx::Ctx<'a, 'ctx>,
    ) -> (Value<'ctx>, Option<String>) {
        let typenode = self.typenode.clone();
        // build debug info
        let param_ditypes = self
            .typenode
            .paralist
            .iter()
            .map(|para| para.deref().tp.get_debug_type(ctx).unwrap())
            .collect::<Vec<_>>();
        let subroutine_type = ctx.dibuilder.create_subroutine_type(
            ctx.diunit.get_file(),
            self.typenode.ret.get_debug_type(ctx),
            &param_ditypes,
            DIFlags::PUBLIC,
        );
        let subprogram = ctx.dibuilder.create_function(
            ctx.discope,
            self.typenode.id.as_str(),
            None,
            ctx.diunit.get_file(),
            self.range.start.line as u32,
            subroutine_type,
            true,
            true,
            self.range.start.line as u32,
            DIFlags::PUBLIC,
            false,
        );
        let para_pltype_ids: Vec<&String> =
            typenode.paralist.iter().map(|para| &para.tp.id).collect();
        // get the para's type vec & copy the para's name vec
        let mut para_names = Vec::new();
        for para in typenode.paralist.iter() {
            para_names.push(para.id.clone());
        }
        // add function
        let func;
        if let Some((fu, _)) = ctx.get_type(typenode.id.as_str()) {
            func = match fu {
                PLType::FN(fu) => fu.fntype,
                _ => panic!("type error"),
            };
        } else {
            panic!("fn not found");
        }
        func.set_subprogram(subprogram);
        ctx.discope = subprogram.as_debug_info_scope();
        ctx.function = Some(func);

        if let Some(body) = self.body.as_mut() {
            // copy para type
            let mut para_tps = Vec::new();
            for i in 0..para_names.len() {
                let para_type = func.get_nth_param(i as u32).unwrap();
                para_tps.push(para_type);
            }
            // add block
            let allocab = ctx.context.append_basic_block(func, "alloc");
            let entry = ctx.context.append_basic_block(func, "entry");
            ctx.position_at_end(entry);
            // alloc para
            for (i, para) in para_tps.iter_mut().enumerate() {
                let alloca = alloc(ctx, para.get_type(), &para_names[i]);
                // add alloc var debug info
                let divar = ctx.dibuilder.create_parameter_variable(
                    ctx.discope,
                    para_names[i].as_str(),
                    i as u32,
                    ctx.diunit.get_file(),
                    self.range.start.line as u32,
                    param_ditypes[i],
                    false,
                    DIFlags::PUBLIC,
                );
                ctx.build_dbg_location(self.typenode.paralist[i].range.start);
                ctx.dibuilder.insert_declare_at_end(
                    alloca,
                    Some(divar),
                    None,
                    ctx.builder.get_current_debug_location().unwrap(),
                    allocab,
                );
                ctx.builder.build_store(alloca, *para);
                ctx.add_symbol(para_names[i].clone(), alloca, para_pltype_ids[i].clone());
            }
            // emit body
            body.emit(ctx);
            ctx.nodebug_builder.position_at_end(allocab);
            ctx.nodebug_builder.build_unconditional_branch(entry);
            return (Value::None, None);
        }
        (Value::None, None)
    }
}

#[range]
pub struct FuncCallNode {
    pub id: String,
    pub paralist: Vec<Box<dyn Node>>,
}

impl Node for FuncCallNode {
    fn print(&self, tabs: usize, end: bool, mut line: Vec<bool>) {
        deal_line(tabs, &mut line, end);
        tab(tabs, line.clone(), end);
        println!("FuncCallNode");
        let mut i = self.paralist.len();
        tab(tabs + 1, line.clone(), i == 0);
        println!("id: {}", self.id);
        for para in &self.paralist {
            i -= 1;
            para.print(tabs + 1, i == 0, line.clone());
        }
    }
    fn emit<'a, 'ctx>(&'a mut self, ctx: &mut Ctx<'a, 'ctx>) -> (Value<'ctx>, Option<String>) {
        let mut para_values = Vec::new();
        for para in self.paralist.iter_mut() {
            let (v, _) = para.emit(ctx);
            let load = ctx.try_load(v);
            para_values.push(load.as_basic_value_enum().into());
        }
        let func = ctx.module.get_function(self.id.as_str()).unwrap();
        let ret = ctx.builder.build_call(
            func,
            &para_values,
            format(format_args!("call_{}", self.id)).as_str(),
        );
        if let (PLType::FN(fv), _) = ctx.get_type(&self.id).unwrap() {
            match (ret.try_as_basic_value().left(), fv.ret_pltype.as_ref()) {
                (Some(v), Some(pltype)) => return (Value::LoadValue(v), Some(pltype.clone())),
                (None, Some(pltype)) => return (Value::None, Some(pltype.clone())),
                _ => todo!(),
            }
        }
        todo!();
    }
}
#[derive(Clone)]
pub struct FuncTypeNode {
    pub id: String,
    pub paralist: Vec<Box<TypedIdentifierNode>>,
    pub ret: Box<TypeNameNode>,
}
impl FuncTypeNode {
    pub fn emit_func_type<'a, 'ctx>(
        &'a mut self,
        ctx: &mut crate::ast::ctx::Ctx<'a, 'ctx>,
    ) -> FunctionValue<'ctx> {
        if let Some((func, _)) = ctx.get_type(self.id.as_str()) {
            let f = match func {
                PLType::FN(func) => func.fntype,
                _ => panic!("type error"),
            };
            return f;
        }
        let mut para_types = Vec::new();
        for para in self.paralist.iter() {
            para_types.push(ctx.get_type(&para.tp.id).unwrap().0.get_basic_type().into());
        }
        let ret_type = ctx.get_type(&self.ret.id).unwrap().0.get_ret_type();
        let func_type = ret_type.fn_type(&para_types, false);
        let func = ctx.module.add_function(self.id.as_str(), func_type, None);
        ctx.add_type(
            self.id.clone(),
            PLType::FN(FNType {
                name: self.id.clone(),
                fntype: func,
                ret_pltype: Some(self.ret.id.clone()),
            }),
        );
        func
    }
}