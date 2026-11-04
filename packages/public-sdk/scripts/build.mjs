import {copyFile,mkdir,readFile,readdir,writeFile} from 'node:fs/promises'
import {dirname,resolve} from 'node:path'
import {fileURLToPath} from 'node:url'
import {build} from 'esbuild'
import ts from 'typescript'

const root=resolve(dirname(fileURLToPath(import.meta.url)),'../../..')
const source=resolve(root,'contracts/browser')
const outputIndex=process.argv.indexOf('--output')
const output=outputIndex<0?resolve(root,'packages/public-sdk/dist'):resolve(process.argv[outputIndex+1])
await mkdir(output,{recursive:true})
const generated=resolve(source,'generated')
const sources=['dto.ts','operations.ts','captureLayout.ts','publicContractNames.ts','schemaAliases.ts'].map(name=>resolve(generated,name))
await build({entryPoints:sources,outdir:output,bundle:false,format:'esm',platform:'browser',target:'es2022',logLevel:'silent'})
const options={declaration:true,emitDeclarationOnly:true,outDir:output,rootDir:generated,target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext,moduleResolution:ts.ModuleResolutionKind.Bundler,skipLibCheck:true,strict:true}
const program=ts.createProgram(sources,options)
const diagnostics=ts.getPreEmitDiagnostics(program)
if(diagnostics.length)throw Error(ts.formatDiagnosticsWithColorAndContext(diagnostics,{getCurrentDirectory:()=>root,getCanonicalFileName:p=>p,getNewLine:()=> '\n'}))
if(program.emit().emitSkipped)throw Error('SDK declaration emit failed')
for(const name of await readdir(generated))if(name.startsWith('validators.'))await copyFile(resolve(generated,name),resolve(output,name))
for(const name of await readdir(output))if(name.endsWith('.d.ts')){
  const path=resolve(output,name)
  await writeFile(path,(await readFile(path,'utf8')).replaceAll("from './dto'","from './dto.js'"),'utf8')
}
for(const folder of ['data','schemas']){
  await mkdir(resolve(output,folder),{recursive:true})
  for(const name of await readdir(resolve(source,folder)))if(name.endsWith('.json'))await copyFile(resolve(source,folder,name),resolve(output,folder,name))
}
await mkdir(resolve(output,'fixtures'),{recursive:true})
for(const name of await readdir(resolve(source,'fixtures')))await copyFile(resolve(source,'fixtures',name),resolve(output,'fixtures',name))
await writeFile(resolve(output,'index.js'),"export {apiOperations} from './operations.js'\nexport {captureLayout} from './captureLayout.js'\nexport {publicContractNames} from './publicContractNames.js'\nexport {schemaUriAliases,canonicalSchemaUri} from './schemaAliases.js'\n",'utf8')
await writeFile(resolve(output,'index.d.ts'),"export type * from './dto.js'\nexport * from './operations.js'\nexport * from './captureLayout.js'\nexport * from './publicContractNames.js'\nexport * from './schemaAliases.js'\n",'utf8')
console.log('PUBLIC_SDK_BUILD_OK')
