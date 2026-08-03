import type { Density } from './settings-store';
export function DensityToggle({value,onChange}:{value:Density;onChange:(value:Density)=>void}){return <fieldset><legend>Row density</legend>{(['compact','comfortable','spacious'] as const).map(density=><label key={density}><input type="radio" name="density" checked={value===density} onChange={()=>onChange(density)}/>{density}</label>)}</fieldset>}
