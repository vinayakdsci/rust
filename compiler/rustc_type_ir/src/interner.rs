use rustc_ast_ir::Movability;
use rustc_index::bit_set::BitSet;
use smallvec::SmallVec;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

use crate::fold::TypeFoldable;
use crate::inherent::*;
use crate::ir_print::IrPrint;
use crate::lang_items::TraitSolverLangItem;
use crate::relate::Relate;
use crate::solve::inspect::CanonicalGoalEvaluationStep;
use crate::solve::{ExternalConstraintsData, PredefinedOpaquesData, SolverMode};
use crate::visit::{Flags, TypeSuperVisitable, TypeVisitable};
use crate::{self as ty};

pub trait Interner:
    Sized
    + Copy
    + IrPrint<ty::AliasTy<Self>>
    + IrPrint<ty::AliasTerm<Self>>
    + IrPrint<ty::TraitRef<Self>>
    + IrPrint<ty::TraitPredicate<Self>>
    + IrPrint<ty::ExistentialTraitRef<Self>>
    + IrPrint<ty::ExistentialProjection<Self>>
    + IrPrint<ty::ProjectionPredicate<Self>>
    + IrPrint<ty::NormalizesTo<Self>>
    + IrPrint<ty::SubtypePredicate<Self>>
    + IrPrint<ty::CoercePredicate<Self>>
    + IrPrint<ty::FnSig<Self>>
{
    type DefId: DefId<Self>;
    type LocalDefId: Copy + Debug + Hash + Eq + Into<Self::DefId> + TypeFoldable<Self>;

    type GenericArgs: GenericArgs<Self>;
    type GenericArgsSlice: Copy + Debug + Hash + Eq + SliceLike<Item = Self::GenericArg>;
    type GenericArg: GenericArg<Self>;
    type Term: Term<Self>;

    type BoundVarKinds: Copy + Debug + Hash + Eq + SliceLike<Item = Self::BoundVarKind> + Default;
    type BoundVarKind: Copy + Debug + Hash + Eq;

    type PredefinedOpaques: Copy
        + Debug
        + Hash
        + Eq
        + TypeFoldable<Self>
        + Deref<Target = PredefinedOpaquesData<Self>>;
    fn mk_predefined_opaques_in_body(
        self,
        data: PredefinedOpaquesData<Self>,
    ) -> Self::PredefinedOpaques;

    type DefiningOpaqueTypes: Copy
        + Debug
        + Hash
        + Default
        + Eq
        + TypeVisitable<Self>
        + SliceLike<Item = Self::LocalDefId>;
    type CanonicalGoalEvaluationStepRef: Copy
        + Debug
        + Hash
        + Eq
        + Deref<Target = CanonicalGoalEvaluationStep<Self>>;

    type CanonicalVars: Copy
        + Debug
        + Hash
        + Eq
        + SliceLike<Item = ty::CanonicalVarInfo<Self>>
        + Default;
    fn mk_canonical_var_infos(self, infos: &[ty::CanonicalVarInfo<Self>]) -> Self::CanonicalVars;

    type ExternalConstraints: Copy
        + Debug
        + Hash
        + Eq
        + TypeFoldable<Self>
        + Deref<Target = ExternalConstraintsData<Self>>;
    fn mk_external_constraints(
        self,
        data: ExternalConstraintsData<Self>,
    ) -> Self::ExternalConstraints;

    type DepNodeIndex;
    fn with_cached_task<T>(self, task: impl FnOnce() -> T) -> (T, Self::DepNodeIndex);

    // Kinds of tys
    type Ty: Ty<Self>;
    type Tys: Tys<Self>;
    type FnInputTys: Copy + Debug + Hash + Eq + SliceLike<Item = Self::Ty> + TypeVisitable<Self>;
    type ParamTy: Copy + Debug + Hash + Eq + ParamLike;
    type BoundTy: Copy + Debug + Hash + Eq + BoundVarLike<Self>;
    type PlaceholderTy: PlaceholderLike;

    // Things stored inside of tys
    type ErrorGuaranteed: Copy + Debug + Hash + Eq;
    type BoundExistentialPredicates: BoundExistentialPredicates<Self>;
    type AllocId: Copy + Debug + Hash + Eq;
    type Pat: Copy + Debug + Hash + Eq + Debug + Relate<Self>;
    type Safety: Safety<Self>;
    type Abi: Abi<Self>;

    // Kinds of consts
    type Const: Const<Self>;
    type PlaceholderConst: PlaceholderLike;
    type ParamConst: Copy + Debug + Hash + Eq + ParamLike;
    type BoundConst: Copy + Debug + Hash + Eq + BoundVarLike<Self>;
    type ValueConst: Copy + Debug + Hash + Eq;
    type ExprConst: Copy + Debug + Hash + Eq + Relate<Self>;

    // Kinds of regions
    type Region: Region<Self>;
    type EarlyParamRegion: Copy + Debug + Hash + Eq + ParamLike;
    type LateParamRegion: Copy + Debug + Hash + Eq;
    type BoundRegion: Copy + Debug + Hash + Eq + BoundVarLike<Self>;
    type PlaceholderRegion: PlaceholderLike;

    // Predicates
    type ParamEnv: ParamEnv<Self>;
    type Predicate: Predicate<Self>;
    type Clause: Clause<Self>;
    type Clauses: Copy + Debug + Hash + Eq + TypeSuperVisitable<Self> + Flags;

    type EvaluationCache: EvaluationCache<Self>;
    fn evaluation_cache(self, mode: SolverMode) -> Self::EvaluationCache;

    fn expand_abstract_consts<T: TypeFoldable<Self>>(self, t: T) -> T;

    type GenericsOf: GenericsOf<Self>;
    fn generics_of(self, def_id: Self::DefId) -> Self::GenericsOf;

    type VariancesOf: Copy + Debug + SliceLike<Item = ty::Variance>;
    fn variances_of(self, def_id: Self::DefId) -> Self::VariancesOf;

    fn type_of(self, def_id: Self::DefId) -> ty::EarlyBinder<Self, Self::Ty>;

    type AdtDef: AdtDef<Self>;
    fn adt_def(self, adt_def_id: Self::DefId) -> Self::AdtDef;

    fn alias_ty_kind(self, alias: ty::AliasTy<Self>) -> ty::AliasTyKind;

    fn alias_term_kind(self, alias: ty::AliasTerm<Self>) -> ty::AliasTermKind;

    fn trait_ref_and_own_args_for_alias(
        self,
        def_id: Self::DefId,
        args: Self::GenericArgs,
    ) -> (ty::TraitRef<Self>, Self::GenericArgsSlice);

    fn mk_args(self, args: &[Self::GenericArg]) -> Self::GenericArgs;

    fn mk_args_from_iter<I, T>(self, args: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<Self::GenericArg, Self::GenericArgs>;

    fn check_args_compatible(self, def_id: Self::DefId, args: Self::GenericArgs) -> bool;

    fn debug_assert_args_compatible(self, def_id: Self::DefId, args: Self::GenericArgs);

    fn intern_canonical_goal_evaluation_step(
        self,
        step: CanonicalGoalEvaluationStep<Self>,
    ) -> Self::CanonicalGoalEvaluationStepRef;

    fn mk_type_list_from_iter<I, T>(self, args: I) -> T::Output
    where
        I: Iterator<Item = T>,
        T: CollectAndApply<Self::Ty, Self::Tys>;

    fn parent(self, def_id: Self::DefId) -> Self::DefId;

    fn recursion_limit(self) -> usize;

    type Features: Features<Self>;
    fn features(self) -> Self::Features;

    fn bound_coroutine_hidden_types(
        self,
        def_id: Self::DefId,
    ) -> impl IntoIterator<Item = ty::EarlyBinder<Self, ty::Binder<Self, Self::Ty>>>;

    fn fn_sig(
        self,
        def_id: Self::DefId,
    ) -> ty::EarlyBinder<Self, ty::Binder<Self, ty::FnSig<Self>>>;

    fn coroutine_movability(self, def_id: Self::DefId) -> Movability;

    fn coroutine_for_closure(self, def_id: Self::DefId) -> Self::DefId;

    fn generics_require_sized_self(self, def_id: Self::DefId) -> bool;

    fn item_bounds(
        self,
        def_id: Self::DefId,
    ) -> ty::EarlyBinder<Self, impl IntoIterator<Item = Self::Clause>>;

    fn predicates_of(
        self,
        def_id: Self::DefId,
    ) -> ty::EarlyBinder<Self, impl IntoIterator<Item = Self::Clause>>;

    fn own_predicates_of(
        self,
        def_id: Self::DefId,
    ) -> ty::EarlyBinder<Self, impl IntoIterator<Item = Self::Clause>>;

    fn explicit_super_predicates_of(
        self,
        def_id: Self::DefId,
    ) -> ty::EarlyBinder<Self, impl IntoIterator<Item = Self::Clause>>;

    fn has_target_features(self, def_id: Self::DefId) -> bool;

    fn require_lang_item(self, lang_item: TraitSolverLangItem) -> Self::DefId;

    fn is_lang_item(self, def_id: Self::DefId, lang_item: TraitSolverLangItem) -> bool;

    fn as_lang_item(self, def_id: Self::DefId) -> Option<TraitSolverLangItem>;

    fn associated_type_def_ids(self, def_id: Self::DefId) -> impl IntoIterator<Item = Self::DefId>;

    // FIXME: move `fast_reject` into `rustc_type_ir`.
    fn args_may_unify_deep(
        self,
        obligation_args: Self::GenericArgs,
        impl_args: Self::GenericArgs,
    ) -> bool;

    fn for_each_relevant_impl(
        self,
        trait_def_id: Self::DefId,
        self_ty: Self::Ty,
        f: impl FnMut(Self::DefId),
    );

    fn has_item_definition(self, def_id: Self::DefId) -> bool;

    fn impl_is_default(self, impl_def_id: Self::DefId) -> bool;

    fn impl_trait_ref(self, impl_def_id: Self::DefId) -> ty::EarlyBinder<Self, ty::TraitRef<Self>>;

    fn impl_polarity(self, impl_def_id: Self::DefId) -> ty::ImplPolarity;

    fn trait_is_auto(self, trait_def_id: Self::DefId) -> bool;

    fn trait_is_alias(self, trait_def_id: Self::DefId) -> bool;

    fn trait_is_object_safe(self, trait_def_id: Self::DefId) -> bool;

    fn trait_may_be_implemented_via_object(self, trait_def_id: Self::DefId) -> bool;

    fn fn_trait_kind_from_def_id(self, trait_def_id: Self::DefId) -> Option<ty::ClosureKind>;

    fn async_fn_trait_kind_from_def_id(self, trait_def_id: Self::DefId) -> Option<ty::ClosureKind>;

    fn supertrait_def_ids(self, trait_def_id: Self::DefId)
    -> impl IntoIterator<Item = Self::DefId>;

    fn delay_bug(self, msg: impl ToString) -> Self::ErrorGuaranteed;

    fn is_general_coroutine(self, coroutine_def_id: Self::DefId) -> bool;
    fn coroutine_is_async(self, coroutine_def_id: Self::DefId) -> bool;
    fn coroutine_is_gen(self, coroutine_def_id: Self::DefId) -> bool;
    fn coroutine_is_async_gen(self, coroutine_def_id: Self::DefId) -> bool;

    fn layout_is_pointer_like(self, param_env: Self::ParamEnv, ty: Self::Ty) -> bool;

    type UnsizingParams: Deref<Target = BitSet<u32>>;
    fn unsizing_params_for_adt(self, adt_def_id: Self::DefId) -> Self::UnsizingParams;

    fn find_const_ty_from_env(
        self,
        param_env: Self::ParamEnv,
        placeholder: Self::PlaceholderConst,
    ) -> Self::Ty;
}

/// Imagine you have a function `F: FnOnce(&[T]) -> R`, plus an iterator `iter`
/// that produces `T` items. You could combine them with
/// `f(&iter.collect::<Vec<_>>())`, but this requires allocating memory for the
/// `Vec`.
///
/// This trait allows for faster implementations, intended for cases where the
/// number of items produced by the iterator is small. There is a blanket impl
/// for `T` items, but there is also a fallible impl for `Result<T, E>` items.
pub trait CollectAndApply<T, R>: Sized {
    type Output;

    /// Produce a result of type `Self::Output` from `iter`. The result will
    /// typically be produced by applying `f` on the elements produced by
    /// `iter`, though this may not happen in some impls, e.g. if an error
    /// occurred during iteration.
    fn collect_and_apply<I, F>(iter: I, f: F) -> Self::Output
    where
        I: Iterator<Item = Self>,
        F: FnOnce(&[T]) -> R;
}

/// The blanket impl that always collects all elements and applies `f`.
impl<T, R> CollectAndApply<T, R> for T {
    type Output = R;

    /// Equivalent to `f(&iter.collect::<Vec<_>>())`.
    fn collect_and_apply<I, F>(mut iter: I, f: F) -> R
    where
        I: Iterator<Item = T>,
        F: FnOnce(&[T]) -> R,
    {
        // This code is hot enough that it's worth specializing for the most
        // common length lists, to avoid the overhead of `SmallVec` creation.
        // Lengths 0, 1, and 2 typically account for ~95% of cases. If
        // `size_hint` is incorrect a panic will occur via an `unwrap` or an
        // `assert`.
        match iter.size_hint() {
            (0, Some(0)) => {
                assert!(iter.next().is_none());
                f(&[])
            }
            (1, Some(1)) => {
                let t0 = iter.next().unwrap();
                assert!(iter.next().is_none());
                f(&[t0])
            }
            (2, Some(2)) => {
                let t0 = iter.next().unwrap();
                let t1 = iter.next().unwrap();
                assert!(iter.next().is_none());
                f(&[t0, t1])
            }
            _ => f(&iter.collect::<SmallVec<[_; 8]>>()),
        }
    }
}

/// A fallible impl that will fail, without calling `f`, if there are any
/// errors during collection.
impl<T, R, E> CollectAndApply<T, R> for Result<T, E> {
    type Output = Result<R, E>;

    /// Equivalent to `Ok(f(&iter.collect::<Result<Vec<_>>>()?))`.
    fn collect_and_apply<I, F>(mut iter: I, f: F) -> Result<R, E>
    where
        I: Iterator<Item = Result<T, E>>,
        F: FnOnce(&[T]) -> R,
    {
        // This code is hot enough that it's worth specializing for the most
        // common length lists, to avoid the overhead of `SmallVec` creation.
        // Lengths 0, 1, and 2 typically account for ~95% of cases. If
        // `size_hint` is incorrect a panic will occur via an `unwrap` or an
        // `assert`, unless a failure happens first, in which case the result
        // will be an error anyway.
        Ok(match iter.size_hint() {
            (0, Some(0)) => {
                assert!(iter.next().is_none());
                f(&[])
            }
            (1, Some(1)) => {
                let t0 = iter.next().unwrap()?;
                assert!(iter.next().is_none());
                f(&[t0])
            }
            (2, Some(2)) => {
                let t0 = iter.next().unwrap()?;
                let t1 = iter.next().unwrap()?;
                assert!(iter.next().is_none());
                f(&[t0, t1])
            }
            _ => f(&iter.collect::<Result<SmallVec<[_; 8]>, _>>()?),
        })
    }
}
