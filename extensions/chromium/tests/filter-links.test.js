import { describe, expect, it } from 'vitest'; import { filterLinks } from '../shared/filter-links.js';
describe('extension filtering',()=>it('deduplicates and rejects non-http links',()=>expect(filterLinks(['https://example.com/a','https://example.com/a','javascript:alert(1)'])).toEqual(['https://example.com/a'])));
