// HEADLESS presentation boundaries only. Real FormRuntime parser/reconciler runs.
import React from 'react'
export const Hint=({children})=><span>{children}</span>
export const HintBar=Hint, Kbd=Hint, ViewHeader=Hint
export const NativeColorWheel=()=>null
export const parseNativeColorWheel=()=>null
export const DetailRuntime=()=>null, GridRuntime=()=>null, ListRuntime=()=>null
