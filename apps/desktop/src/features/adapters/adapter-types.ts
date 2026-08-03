export type ResolutionPolicy={cookiePolicy:"refuse";credentialPolicy:"refuse"};
export type ResolvedLink={url:string;displayName?:string;adapterId:string;metadata?:Record<string,string>};
export type HostAdapter={id:string;label:string;matches:(url:URL)=>boolean;resolve:(input:{url:string;policy:ResolutionPolicy})=>Promise<ResolvedLink[]>};
