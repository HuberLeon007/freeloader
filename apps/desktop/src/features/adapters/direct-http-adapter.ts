import type { HostAdapter } from "./adapter-types";
export const directHttpAdapter:HostAdapter={id:"direct-http",label:"Direct HTTP",matches:()=>true,async resolve({url,policy}){if(policy.cookiePolicy!=="refuse"||policy.credentialPolicy!=="refuse")throw new Error("Freeloader refuses cookies and credentials");return [{url,adapterId:"direct-http"}];}};
