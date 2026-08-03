import { describe, expect, it } from 'vitest'; import { FailurePanel } from '../src/features/diagnostics/FailurePanel';
describe('diagnostics contract',()=>it('exports readable failure content',()=>{expect(FailurePanel).toBeTypeOf('function');}));
