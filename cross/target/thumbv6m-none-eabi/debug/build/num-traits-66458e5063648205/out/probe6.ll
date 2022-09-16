; ModuleID = 'probe6.553e6923-cgu.0'
source_filename = "probe6.553e6923-cgu.0"
target datalayout = "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64"
target triple = "thumbv6m-none-unknown-eabi"

%"core::panic::location::Location" = type { { [0 x i8]*, i32 }, i32, i32 }

@alloc3 = private unnamed_addr constant <{ [75 x i8] }> <{ [75 x i8] c"/rustc/4b91a6ea7258a947e59c6522cd5898e7c0a6a88f/library/core/src/num/mod.rs" }>, align 1
@alloc4 = private unnamed_addr constant <{ i8*, [12 x i8] }> <{ i8* getelementptr inbounds (<{ [75 x i8] }>, <{ [75 x i8] }>* @alloc3, i32 0, i32 0, i32 0), [12 x i8] c"K\00\00\00K\03\00\00\05\00\00\00" }>, align 4
@str.0 = internal constant [25 x i8] c"attempt to divide by zero"

; probe6::probe
; Function Attrs: nounwind
define dso_local void @_ZN6probe65probe17h02d85f581e587b28E() unnamed_addr #0 {
start:
  %0 = call i1 @llvm.expect.i1(i1 false, i1 false) #3
  br i1 %0, label %panic.i, label %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17hba51fb66a888e3f8E.exit"

panic.i:                                          ; preds = %start
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17h2b55aa182ea32d92E([0 x i8]* align 1 bitcast ([25 x i8]* @str.0 to [0 x i8]*), i32 25, %"core::panic::location::Location"* align 4 bitcast (<{ i8*, [12 x i8] }>* @alloc4 to %"core::panic::location::Location"*)) #4
  unreachable

"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17hba51fb66a888e3f8E.exit": ; preds = %start
  br label %bb1

bb1:                                              ; preds = %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17hba51fb66a888e3f8E.exit"
  ret void
}

; Function Attrs: nofree nosync nounwind readnone willreturn
declare i1 @llvm.expect.i1(i1, i1) #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn nounwind
declare dso_local void @_ZN4core9panicking5panic17h2b55aa182ea32d92E([0 x i8]* align 1, i32, %"core::panic::location::Location"* align 4) unnamed_addr #2

attributes #0 = { nounwind "frame-pointer"="all" "target-cpu"="generic" "target-features"="+strict-align" }
attributes #1 = { nofree nosync nounwind readnone willreturn }
attributes #2 = { cold noinline noreturn nounwind "frame-pointer"="all" "target-cpu"="generic" "target-features"="+strict-align" }
attributes #3 = { nounwind }
attributes #4 = { noreturn nounwind }
