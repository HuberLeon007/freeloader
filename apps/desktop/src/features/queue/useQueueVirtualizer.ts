import { useVirtualizer } from '@tanstack/react-virtual';
import { useRef } from 'react';
export function useQueueVirtualizer(count:number){const parentRef=useRef<HTMLDivElement>(null);const virtualizer=useVirtualizer({count,getScrollElement:()=>parentRef.current,estimateSize:()=>40,overscan:8,measureElement:element=>element.getBoundingClientRect().height,paddingStart:40,scrollPaddingStart:40});return {parentRef,virtualizer};}
