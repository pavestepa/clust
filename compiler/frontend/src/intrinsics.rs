//! Compiler-owned module. Private symbols cannot be spelled by source programs.
pub const MODULE: &str = "core.intrinsic";
pub const SOURCE: &str = include_str!("intrinsic.cl");
/// Only the embedded intrinsic module may implement an unsafe declaration
/// directly. User declarations continue through the ordinary access checker.
pub(crate) fn lower_trusted_declarations(module: &mut crate::ast::Module) {
    use crate::ast::{Block, Function, ItemKind, Member, Statement, StatementKind};
    fn trusted(body: &mut Block) {
        let span = body.span;
        let statements = std::mem::take(&mut body.statements);
        body.statements.push(Statement {
            kind: StatementKind::Unsafe(Block { statements, span }),
            span,
        });
    }
    fn inline_storage(value: &mut crate::ast::Expr) {
        use crate::ast::{ExprKind, TypeKind};
        let ExprKind::New { ty, arguments } = &mut value.kind else {
            return;
        };
        let TypeKind::Named { path, .. } = &ty.kind else {
            return;
        };
        let fields: &[&str] = match path.last().map(String::as_str) {
            Some("RawStorage") => &["data", "cap"],
            Some("Released") => &["storage"],
            _ => return,
        };
        assert_eq!(fields.len(), arguments.len());
        value.kind = ExprKind::Construct {
            ty: ty.clone(),
            fields: fields
                .iter()
                .zip(std::mem::take(arguments))
                .map(|(name, expr)| ((*name).into(), expr))
                .collect(),
        };
    }
    fn function(f: &mut Function) {
        if let Some(body) = &mut f.body {
            for stmt in &mut body.statements {
                match &mut stmt.kind {
                    StatementKind::Return(Some(value)) | StatementKind::Binding { value, .. } => {
                        inline_storage(value)
                    }
                    _ => {}
                }
            }
        }
        f.unsafe_ = matches!(
            f.name.as_str(),
            "allocate"
                | "from_raw_parts"
                | "check_range"
                | "move_within"
                | "move_from"
                | "initialize"
                | "view"
                | "view_mut"
                | "slot_address"
                | "slot_address_mut"
                | "take"
                | "replace"
                | "drop_range"
                | "drop_slot"
                | "release"
                | "move_slots"
                | "drop_in_place"
                | "into_raw_parts"
                | "read_owned"
                | "write_owned"
        );
        if f.unsafe_ {
            if let Some(body) = &mut f.body {
                trusted(body);
            }
        }
    }
    for item in &mut module.items {
        match &mut item.kind {
            ItemKind::Function(f) => function(f),
            ItemKind::Aggregate { class, members, .. } => {
                *class = false;
                members.retain(|member| !matches!(member, Member::Constructor { .. }));
                for member in members {
                    match member {
                        Member::Method {
                            function: f,
                            dispatch,
                            ..
                        } => {
                            *dispatch = crate::ast::MethodDispatch::Ordinary;
                            function(f);
                        }
                        Member::Destructor(body) => trusted(body),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intrinsic {
    WeakIdentity,
    WeakAlive,
    WeakAccess,
    WriteOwned,
    ReadOwned,
    Detach,
    Adopt,
    DropInPlace,
    SlotAddress,
    SlotAddressMut,
    CheckIndex,
    View,
    ViewMut,
    DropRange,
    MoveSlots,
    CheckRange,
    Allocate,
    Initialize,
    Take,
    Replace,
    DropSlot,
    Release,
    Size,
    Align,
}
impl Intrinsic {
    pub const ALL: [Self; 24] = [
        Self::WeakIdentity,
        Self::WeakAlive,
        Self::WeakAccess,
        Self::WriteOwned,
        Self::ReadOwned,
        Self::Detach,
        Self::Adopt,
        Self::DropInPlace,
        Self::SlotAddress,
        Self::SlotAddressMut,
        Self::CheckIndex,
        Self::View,
        Self::ViewMut,
        Self::DropRange,
        Self::MoveSlots,
        Self::CheckRange,
        Self::Allocate,
        Self::Initialize,
        Self::Take,
        Self::Replace,
        Self::DropSlot,
        Self::Release,
        Self::Size,
        Self::Align,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::WeakIdentity => "weak_identity",
            Self::WeakAlive => "weak_alive",
            Self::WeakAccess => "weak_access",
            Self::WriteOwned => "pointer_initialize",
            Self::ReadOwned => "pointer_take",
            Self::Detach => "heap_detach",
            Self::Adopt => "heap_adopt",
            Self::DropInPlace => "pointer_drop",
            Self::SlotAddress => "heap_address",
            Self::SlotAddressMut => "heap_address_mut",
            Self::CheckIndex => "bounds_index",
            Self::View => "heap_view",
            Self::ViewMut => "heap_view_mut",
            Self::DropRange => "heap_drop_range",
            Self::MoveSlots => "heap_move_slots",
            Self::CheckRange => "heap_check_range",
            Self::Allocate => "heap_allocate",
            Self::Initialize => "heap_initialize",
            Self::Take => "heap_take",
            Self::Replace => "heap_replace",
            Self::DropSlot => "heap_drop",
            Self::Release => "heap_release",
            Self::Size => "layout_size",
            Self::Align => "layout_align",
        }
    }
    pub fn symbol(self) -> &'static str {
        match self {
            Self::WeakIdentity => "$intrinsic.weak_identity",
            Self::WeakAlive => "$intrinsic.weak_alive",
            Self::WeakAccess => "$intrinsic.weak_access",
            Self::WriteOwned => "$intrinsic.pointer_initialize",
            Self::ReadOwned => "$intrinsic.pointer_take",
            Self::Detach => "$intrinsic.heap_detach",
            Self::Adopt => "$intrinsic.heap_adopt",
            Self::DropInPlace => "$intrinsic.pointer_drop",
            Self::SlotAddress => "$intrinsic.heap_address",
            Self::SlotAddressMut => "$intrinsic.heap_address_mut",
            Self::CheckIndex => "$intrinsic.bounds_index",
            Self::View => "$intrinsic.heap_view",
            Self::ViewMut => "$intrinsic.heap_view_mut",
            Self::DropRange => "$intrinsic.heap_drop_range",
            Self::MoveSlots => "$intrinsic.heap_move_slots",
            Self::CheckRange => "$intrinsic.heap_check_range",
            Self::Allocate => "$intrinsic.heap_allocate",
            Self::Initialize => "$intrinsic.heap_initialize",
            Self::Take => "$intrinsic.heap_take",
            Self::Replace => "$intrinsic.heap_replace",
            Self::DropSlot => "$intrinsic.heap_drop",
            Self::Release => "$intrinsic.heap_release",
            Self::Size => "$intrinsic.layout_size",
            Self::Align => "$intrinsic.layout_align",
        }
    }
}

mod resolution;
