// HEADLESS presentation: forwards click to the real Surface onExecute callback.
export const ActionPanelOverlay=({actions,onExecute})=><div>{actions.map(a=><button key={a.id} data-action={a.id} onClick={()=>onExecute(a)}>{a.title}</button>)}</div>
