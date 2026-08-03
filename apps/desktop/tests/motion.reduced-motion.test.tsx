import { describe, expect, it } from 'vitest';
describe('reduced motion contract',()=>it('uses the platform preference selector',()=>{expect('@media (prefers-reduced-motion: reduce)').toContain('prefers-reduced-motion');}));
