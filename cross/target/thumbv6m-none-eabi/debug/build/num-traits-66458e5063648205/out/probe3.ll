; ModuleID = 'probe3.cb64369a-cgu.0'
source_filename = "probe3.cb64369a-cgu.0"
target datalayout = "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64"
target triple = "thumbv6m-none-unknown-eabi"

; probe3::probe
; Function Attrs: nounwind
define dso_local void @_ZN6probe35probe17h2b3c9b1031ab9009E() unnamed_addr #0 {
start:
  %0 = alloca i32, align 4
  store i32 -2147483648, i32* %0, align 4
  %1 = load i32, i32* %0, align 4
  br label %bb1

bb1:                                              ; preds = %start
  ret void
}

; Function Attrs: nofree nosync nounwind readnone speculatable willreturn
declare i32 @llvm.bitreverse.i32(i32) #1

attributes #0 = { nounwind "frame-pointer"="all" "target-cpu"="generic" "target-features"="+strict-align" }
attributes #1 = { nofree nosync nounwind readnone speculatable willreturn }
