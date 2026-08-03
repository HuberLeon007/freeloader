import { describe, expect, it } from 'vitest'; import { filterLinks } from '../shared/filter-links.js';
describe('extension filtering',()=>it('keeps only web links',()=>expect(filterLinks(['https://example.com','file:///tmp/a'])).toEqual(['https://example.com'])));
