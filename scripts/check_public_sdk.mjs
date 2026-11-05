// Inspect actual packed bytes and execute the extracted release without tools.
import {mkdir,readFile,readdir,writeFile,copyFile} from 'node:fs/promises'
import {execFileSync} from 'node:child_process'
import {dirname,resolve} from 'node:path'
import {fileURLToPath,pathToFileURL} from 'node:url'
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..')
const output=process.argv[process.argv.indexOf('--output')+1]
if(!output||!process.argv.includes('--output'))throw Error('An owned external --output is required')
const {validateArtifactDirectory}=await import(process.env.CIRNET_ARTIFACT_PATHS_MODULE??process.env.NETCIRCUIT_ARTIFACT_PATHS_MODULE??'@cirnet/artifact-paths')
validateArtifactDirectory(resolve(output))
await mkdir(output,{recursive:true})
const packageStage=resolve(output,'sdk-source')
await mkdir(packageStage,{recursive:true})
for(const name of ['package.json','README.md','LICENSE'])await copyFile(resolve(root,'packages/public-sdk',name),resolve(packageStage,name))
execFileSync('node',[resolve(root,'packages/public-sdk/scripts/build.mjs'),'--output',resolve(packageStage,'dist')],{cwd:root,stdio:'pipe'})
const npmArguments=['pack','--json','--ignore-scripts','--cache',resolve(output,'npm-cache'),'--pack-destination',resolve(output)]
const npm=process.platform==='win32'?process.execPath:'npm'
if(process.platform==='win32')npmArguments.unshift(resolve(dirname(process.execPath),'node_modules/npm/bin/npm-cli.js'))
const packResult=JSON.parse(execFileSync(npm,npmArguments,{cwd:packageStage,encoding:'utf8'}))
const packed=Array.isArray(packResult)?packResult[0]:Object.values(packResult)[0]
if(!packed?.files||!packed.filename)throw Error('Unexpected npm pack metadata shape')
for(const file of packed.files){
  if(!['package.json','README.md','LICENSE'].includes(file.path)&&!file.path.startsWith('dist/'))throw Error('Unexpected tarball file: '+file.path)
  if(/(?:catalog-source|compatibility|rpc_v2|\.proto$)/.test(file.path))throw Error('Internal resource leaked: '+file.path)
}
// npm's listing is checked against the extracted tarball contents too.
execFileSync('tar',['-xzf',resolve(output,packed.filename),'-C',resolve(output)],{stdio:'pipe'})
const unpacked=resolve(output,'package')
const privateIdentifiers=['InstrumentationExecutionPlan','StationCapabilities','FpgaControlScaffold']
async function inspect(path){
  for(const item of await readdir(path,{withFileTypes:true})){
    const child=resolve(path,item.name)
    if(item.isDirectory())await inspect(child)
    else{
      let value=await readFile(child,'utf8')
      // A public projection comment explains the internal type it replaces;
      // that annotation is not a leaked definition or descendant reference.
      if(child.endsWith('.json'))value=JSON.stringify(JSON.parse(value),(key,item)=>['$comment','description'].includes(key)?undefined:item)
      if(child.endsWith('.js'))value=value.replace(/(?:"\$comment"|\$comment|"description"|description)\s*:\s*"(?:[^"\\]|\\.)*"/g,'')
      for(const name of privateIdentifiers)if(new RegExp('(?<![A-Za-z0-9_])'+name+'(?![A-Za-z0-9_])').test(value))throw Error('Private identifier leaked in '+child+': '+name)
    }
  }
}
await inspect(resolve(unpacked,'dist'))
const probe=resolve(output,'probe.mjs')
await writeFile(probe,`import assert from 'node:assert/strict'\nimport {createRequire} from 'node:module'\nconst require=createRequire(import.meta.url)\nconst sdk=await import(${JSON.stringify(pathToFileURL(resolve(unpacked,'dist/index.js')).href)})\nconst validators=await import(${JSON.stringify(pathToFileURL(resolve(unpacked,'dist/validators.js')).href)})\nassert.equal(sdk.canonicalSchemaUri('https://netcircuit.local/contracts/public/api/1.0/schema.json#/$defs/HealthResponse'),'https://cirnet.local/contracts/public/api/1.0/schema.json#/$defs/HealthResponse')\nassert(validators.HealthResponse({status:'ok',service:'cirnet-api',hardware_mode:'simulation'}))\nassert(validators.HealthResponse({status:'ok',service:'netcircuit-api',hardware_mode:'simulation'}))\nassert(sdk.publicContractNames.includes('CircuitGraph'))\nassert(!sdk.publicContractNames.includes('InstrumentationExecutionPlan'))\nassert(validators.DigitalObservation({signal:'out',time_ns:0,value:0}))\nassert(!validators.DigitalObservation({signal:'out',time_ns:0,value:2}))\nassert(Object.keys(sdk.apiOperations).length>0)\nassert.throws(()=>require.resolve('ajv'))\nconsole.log('EXTRACTED_SDK_RUNTIME_OK')\n`)
execFileSync('node',[probe],{cwd:resolve(output),stdio:'pipe'})
console.log('PUBLIC_SDK_PACK_OK '+packed.filename+' '+packed.integrity)
