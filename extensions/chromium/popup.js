import { filterLinks } from './shared/filter-links.js';
const links=document.querySelector('#links');const status=document.querySelector('#status');document.querySelector('#send').addEventListener('click',()=>{const values=filterLinks(links.value.split(/\r?\n/));status.textContent=`${values.length} links ready`;});
