export function scanPage(){return Array.from(document.querySelectorAll('a[href]')).map(anchor=>anchor.href).filter(Boolean);}
