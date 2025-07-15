use std::{cell::RefCell, collections::HashMap, io::Write, ops::Index};

use cfg_resolution::{PseudoASTTag as PT, ResolvedBlock};

use crate::bytecode::{
    defs::{Instr, StackItem},
    symbolic_evaluation::{AnnotatedBlock, BasicBlockToken, ConditionalJump},
};

mod cfg_resolution;

fn write_indented<'a>(writer: &mut impl Write, args: std::fmt::Arguments<'a>, indent_depth: usize) {
    for _ in 0..indent_depth {
        let _ = write!(writer, "\t");
    }

    let _ = writer.write_fmt(args);
}

struct Context<'a, 'b, W: Write> {
    writer: &'a RefCell<W>,
    graph: &'b HashMap<BasicBlockToken, ResolvedBlock>,
    depth: usize,
}

impl<'a, 'b, W: Write> Clone for Context<'a, 'b, W> {
    fn clone(&self) -> Self {
        Context {
            writer: self.writer,
            graph: self.graph,
            depth: self.depth,
        }
    }
}

impl<'a, 'b, W: Write> Copy for Context<'a, 'b, W> {}

impl<'a, 'b, W: Write> Index<&BasicBlockToken> for Context<'a, 'b, W> {
    type Output = ResolvedBlock;
    fn index(&self, index: &BasicBlockToken) -> &Self::Output {
        &self.graph[index]
    }
}

// I'd like to rework this into something like
// <https://mcyoung.xyz/2025/03/11/formatters/>
pub fn gen_code(graph: &HashMap<BasicBlockToken, AnnotatedBlock>, writer: impl Write) {
    let resolved_map = RefCell::new(HashMap::new());
    graph
        .keys()
        .for_each(|token| cfg_resolution::resolve_tags(*token, graph, &resolved_map));
    let resolved_map = resolved_map.into_inner();
    // println!("{resolved_map:#?}");

    let ctx = Context {
        writer: &RefCell::new(writer),
        graph: &resolved_map,
        depth: 0,
    };

    // println!("Resolving block at 0");
    for_block(&resolved_map[&BasicBlockToken::zero()], ctx);
}

fn for_block<'a, 'b, W: Write>(block: &ResolvedBlock, ctx: Context<'a, 'b, W>) {
    match block {
        ResolvedBlock {
            ast_tag:
                PT::ForLoop {
                    body,
                    falls_through_to,
                    assignment,
                },
            ..
        } => {
            if let Instr::StoreFast(name, val) = assignment {
                write_indented(
                    &mut *ctx.writer.borrow_mut(),
                    format_args!("for {name} in "),
                    ctx.depth,
                );
                for_stack_item(val, ctx);
                let _ = write!(&mut *ctx.writer.borrow_mut(), ":\n");
                let mut deeper = ctx.clone();
                deeper.depth += 1;
                // println!("Resolving block at {body:?}");
                for_block(&ctx[body], deeper);
                // println!("Resolving block at {falls_through_to:?}");
                for_block(&ctx[falls_through_to], ctx);
            } else {
                // Strictly speaking this is not sufficient, but for now is fine
                panic!("Expected a store in the for loop header")
            }
        }
        ResolvedBlock {
            ast_tag: PT::FallsThrough(to),
            body,
        } => {
            body.iter().for_each(|instr| for_instr(instr, ctx, true));
            // println!("Resolving block at {to:?}");
            for_block(&ctx[to], ctx);
        }
        ResolvedBlock {
            ast_tag: PT::Breaks,
            body,
        } => {
            body.iter().for_each(|instr| for_instr(instr, ctx, true));
            write_indented(
                &mut *ctx.writer.borrow_mut(),
                format_args!("break\n"),
                ctx.depth,
            );
        }
        ResolvedBlock {
            ast_tag: PT::Continues,
            body,
        } => {
            body.iter().for_each(|instr| for_instr(instr, ctx, true));
            write_indented(
                &mut *ctx.writer.borrow_mut(),
                format_args!("continue\n"),
                ctx.depth,
            );
        }
        ResolvedBlock {
            ast_tag:
                PT::WhileHead {
                    jump,
                    body,
                    falls_through_to,
                },
            ..
        } => {
            write_indented(
                &mut *ctx.writer.borrow_mut(),
                format_args!("while "),
                ctx.depth,
            );
            for_stack_item(&jump.cond, ctx);
            let _ = write!(&mut *ctx.writer.borrow_mut(), ":\n");
            let mut deeper = ctx.clone();
            deeper.depth += 1;
            // println!("Resolving block at {body:?}");
            for_block(&ctx[body], deeper);
            // println!("Resolving block at {falls_through_to:?}");
            for_block(&ctx[falls_through_to], ctx);
        }
        ResolvedBlock {
            ast_tag:
                PT::BareIf {
                    jump,
                    body,
                    falls_through_to,
                },
            ..
        } => {
            handle_if(jump, *body, *falls_through_to, ctx, false);
        }
        ResolvedBlock {
            ast_tag:
                PT::IfElse {
                    jump,
                    body,
                    r#else,
                    falls_through_to,
                },
            ..
        } => {
            handle_if_else(jump, *body, *r#else, *falls_through_to, ctx, false);
        }
        ResolvedBlock {
            ast_tag: PT::Returns(item),
            body,
        } => {
            body.iter().for_each(|instr| for_instr(instr, ctx, true));
            write_indented(
                &mut *ctx.writer.borrow_mut(),
                format_args!("return "),
                ctx.depth,
            );
            for_stack_item(item, ctx);
            let _ = write!(ctx.writer.borrow_mut(), "\n");
        }
        ResolvedBlock {
            body,
            ast_tag: PT::Passes,
        } => {
            body.iter().for_each(|instr| for_instr(instr, ctx, true));
        }
    }
}

fn handle_if_else<'a, 'b, W: Write>(
    jump: &ConditionalJump,
    body: BasicBlockToken,
    r#else: BasicBlockToken,
    falls_through_to: BasicBlockToken,
    ctx: Context<'a, 'b, W>,
    is_else: bool,
) {
    if is_else {
        write_indented(
            &mut *ctx.writer.borrow_mut(),
            format_args!("elif "),
            ctx.depth,
        );
    } else {
        write_indented(
            &mut *ctx.writer.borrow_mut(),
            format_args!("if "),
            ctx.depth,
        );
    }
    for_stack_item(&jump.cond, ctx.clone());
    let _ = write!(&mut *ctx.writer.borrow_mut(), ":\n");
    let mut deeper = ctx.clone();
    deeper.depth += 1;
    // println!("Resolving block at {body:?}");
    for_block(&ctx[&body], deeper);
    match &(ctx.clone())[&r#else] {
        ResolvedBlock {
            ast_tag:
                PT::IfElse {
                    jump,
                    body,
                    r#else,
                    falls_through_to,
                },
            ..
        } => {
            handle_if_else(jump, *body, *r#else, *falls_through_to, ctx.clone(), true);
        }
        ResolvedBlock {
            ast_tag:
                PT::BareIf {
                    jump,
                    body,
                    falls_through_to,
                },
            ..
        } => {
            handle_if(&jump, *body, *falls_through_to, ctx, true);
        }
        block => {
            write_indented(
                &mut *ctx.writer.borrow_mut(),
                format_args!("else:\n"),
                ctx.depth,
            );
            let mut deeper = ctx.clone();
            deeper.depth += 1;
            // println!("Resolving block at {block:?}");
            for_block(&block, deeper);
            // println!("Resolving block at {falls_through_to:?}");
            for_block(&ctx[&falls_through_to], ctx.clone());
        }
    }
}

fn handle_if<'a, 'b, W: Write>(
    jump: &ConditionalJump,
    body: BasicBlockToken,
    falls_through_to: BasicBlockToken,
    ctx: Context<'a, 'b, W>,
    is_else: bool,
) {
    if is_else {
        write_indented(
            &mut *ctx.writer.borrow_mut(),
            format_args!("elif "),
            ctx.depth,
        );
    } else {
        write_indented(
            &mut *ctx.writer.borrow_mut(),
            format_args!("if "),
            ctx.depth,
        );
    }
    for_stack_item(&jump.cond, ctx.clone());
    let _ = write!(&mut *ctx.writer.borrow_mut(), ":\n");
    let mut deeper = ctx.clone();
    deeper.depth += 1;
    // println!("Resolving block at {body:?}");
    for_block(&ctx[&body], deeper);
    // println!("Resolving block at {falls_through_to:?}");
    for_block(&ctx[&falls_through_to], ctx.clone());
}

fn for_stack_item<'a, 'b, W: Write>(item: &StackItem, ctx: Context<'a, 'b, W>) {
    match item {
        StackItem::Derived(instr) => for_instr(&*instr, ctx, false),
        StackItem::Local(name) | StackItem::Global(name) => {
            let _ = write!(ctx.writer.borrow_mut(), "{name}");
        }
        StackItem::Const(item) => {
            let _ = write!(ctx.writer.borrow_mut(), "{}", item.emit_code());
        }
        StackItem::Null | StackItem::DummyIter => {}
    }
}
fn for_instr<'a, 'b, W: Write>(instr: &Instr, ctx: Context<'a, 'b, W>, top_level: bool) {
    use Instr::*;
    if top_level {
        let mut r = ctx.writer.borrow_mut();
        for _ in 0..ctx.depth {
            let _ = write!(r, "\t");
        }
    }
    match instr {
        StoreFast(name, item) => {
            let _ = write!(ctx.writer.borrow_mut(), "{name} = ");
            for_stack_item(item, ctx);
        }
        StoreGlobal(name, item) => {
            let _ = write!(ctx.writer.borrow_mut(), "{name} = ");
            for_stack_item(item, ctx);
        }
        Call {
            obj: StackItem::Null,
            meth,
            args,
        } => {
            for_stack_item(meth, ctx);
            let _ = write!(ctx.writer.borrow_mut(), "(");
            let mut it = args.iter();
            if let Some(arg) = it.next() {
                for_stack_item(arg, ctx);
            }
            for arg in it {
                let _ = write!(ctx.writer.borrow_mut(), ", ");
                for_stack_item(arg, ctx);
            }
            let _ = write!(ctx.writer.borrow_mut(), ")");
        }
        CompareOp(op, lhs, rhs) => {
            for_stack_item(lhs, ctx);
            let _ = write!(ctx.writer.borrow_mut(), " {op} ");
            for_stack_item(rhs, ctx);
        }
        BinaryOp(op, lhs, rhs) => {
            for_stack_item(lhs, ctx);
            let _ = write!(ctx.writer.borrow_mut(), " {op} ");
            for_stack_item(rhs, ctx);
        }
        ForIterNext(item) | GetIter(item) | ToBool(item) => for_stack_item(item, ctx),
        instr => todo!("Haven't implemented {instr:?}"),
    }
    if top_level {
        let _ = write!(ctx.writer.borrow_mut(), "\n");
    }
}
