// HEADLESS hook boundary. No IPC, HUD, clipboard, window or browser effects.
import {createContext} from 'react'
export const NavigationContext=createContext({push(){},pop(){}})
export const setRuntimeContext=()=>{}
export const Clipboard={}, environment={}
export const getPreferenceValues=()=>({}), open=()=>{}, showHUD=()=>{}, showToast=()=>{}
export const useCachedPromise=()=>{},useFetch=()=>{},useNavigation=()=>{},usePromise=()=>{}
