// Inspect actual packed bytes and execute the extracted release without tools.
import assert from 'node:assert/strict'
import {constants,copyFile,lstat,mkdir,mkdtemp,readFile,readdir,writeFile} from 'node:fs/promises'
import {execFileSync} from 'node:child_process'
import {basename,dirname,resolve} from 'node:path'
import {fileURLToPath,pathToFileURL} from 'node:url'
import {parseArgs} from 'node:util'
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..')
const {values}=parseArgs({options:{output:{type:'string'},source:{type:'string'},'node-tools':{type:'string'},baseline:{type:'string'}}})
if(!values.output)throw Error('An owned external --output is required')
const {validateArtifactDirectory}=await import(process.env.CIRNET_ARTIFACT_PATHS_MODULE??process.env.NETCIRCUIT_ARTIFACT_PATHS_MODULE??'@cirnet/artifact-paths')
const output=validateArtifactDirectory(resolve(values.output))
await mkdir(output,{recursive:true})
const run=await mkdtemp(resolve(output,'sdk-pack-'))
const packageStage=resolve(run,'sdk-source')
await mkdir(packageStage)
for(const name of ['package.json','README.md','LICENSE'])await copyFile(resolve(root,'packages/public-sdk',name),resolve(packageStage,name))
const buildArgs=[resolve(root,'packages/public-sdk/scripts/build.mjs'),'--output',resolve(packageStage,'dist')]
for(const option of ['source','node-tools'])if(values[option])buildArgs.push('--'+option,resolve(values[option]))
execFileSync(process.execPath,buildArgs,{cwd:root,stdio:'pipe'})
const npmArguments=['pack','--json','--ignore-scripts','--cache',resolve(run,'npm-cache'),'--pack-destination',run]
const npm=process.platform==='win32'?process.execPath:'npm'
if(process.platform==='win32')npmArguments.unshift(resolve(dirname(process.execPath),'node_modules/npm/bin/npm-cli.js'))
const packResult=JSON.parse(execFileSync(npm,npmArguments,{cwd:packageStage,encoding:'utf8'}))
const packed=Array.isArray(packResult)?packResult[0]:Object.values(packResult)[0]
if(!packed?.files||!packed.filename||basename(packed.filename)!==packed.filename)throw Error('Unexpected npm pack metadata shape')
const archive=resolve(run,packed.filename)
for(const file of packed.files){
  if(!['package.json','README.md','LICENSE'].includes(file.path)&&!file.path.startsWith('dist/'))throw Error('Unexpected tarball file: '+file.path)
  if(/(?:catalog-source|compatibility|rpc_v2|\.proto$)/.test(file.path))throw Error('Internal resource leaked: '+file.path)
}
// npm's listing is checked against the extracted tarball contents too.
execFileSync('tar',['-xzf',archive,'-C',run],{stdio:'pipe'})
const unpacked=resolve(run,'package')
const privateIdentifiers=['InstrumentationExecutionPlan','StationCapabilities','FpgaControlScaffold']
async function requireDirectory(path){
  if(!(await lstat(path)).isDirectory())throw Error('Expected a packed directory: '+path)
}
async function inspect(path){
  for(const item of await readdir(path,{withFileTypes:true})){
    const child=resolve(path,item.name)
    if(item.isDirectory())await inspect(child)
    else if(item.isFile()){
      let value=await readFile(child,'utf8')
      // A public projection comment explains the internal type it replaces;
      // that annotation is not a leaked definition or descendant reference.
      if(child.endsWith('.json'))value=JSON.stringify(JSON.parse(value),(key,item)=>['$comment','description'].includes(key)?undefined:item)
      if(child.endsWith('.js'))value=value.replace(/(?:"\$comment"|\$comment|"description"|description)\s*:\s*"(?:[^"\\]|\\.)*"/g,'')
      for(const name of privateIdentifiers)if(new RegExp('(?<![A-Za-z0-9_])'+name+'(?![A-Za-z0-9_])').test(value))throw Error('Private identifier leaked in '+child+': '+name)
    }else throw Error('Unexpected packed entry: '+child)
  }
}
await requireDirectory(unpacked)
await requireDirectory(resolve(unpacked,'dist'))
await inspect(resolve(unpacked,'dist'))
const probe=resolve(run,'probe.mjs')
await writeFile(probe,`import assert from 'node:assert/strict'\nimport {createRequire} from 'node:module'\nconst require=createRequire(import.meta.url)\nconst sdk=await import(${JSON.stringify(pathToFileURL(resolve(unpacked,'dist/index.js')).href)})\nconst validators=await import(${JSON.stringify(pathToFileURL(resolve(unpacked,'dist/validators.js')).href)})\nassert.equal(sdk.canonicalSchemaUri('https://netcircuit.local/contracts/public/api/1.0/schema.json#/$defs/HealthResponse'),'https://cirnet.local/contracts/public/api/1.0/schema.json#/$defs/HealthResponse')\nassert(validators.HealthResponse({status:'ok',service:'cirnet-api',hardware_mode:'simulation'}))\nassert(validators.HealthResponse({status:'ok',service:'netcircuit-api',hardware_mode:'simulation'}))\nassert(sdk.publicContractNames.includes('CircuitGraph'))\nassert(!sdk.publicContractNames.includes('InstrumentationExecutionPlan'))\nassert(validators.DigitalObservation({signal:'out',time_ns:0,value:0}))\nassert(!validators.DigitalObservation({signal:'out',time_ns:0,value:2}))\nassert(Object.keys(sdk.apiOperations).length>0)\nassert.throws(()=>require.resolve('ajv'))\nconsole.log('EXTRACTED_SDK_RUNTIME_OK')\n`)
execFileSync(process.execPath,[probe],{cwd:run,stdio:'pipe'})
if(values.baseline){
  const sealed=resolve(run,'sealed-sdk')
  await mkdir(sealed)
  execFileSync('tar',['-xzf',resolve(values.baseline),'-C',sealed],{stdio:'pipe'})
  const baseline=resolve(sealed,'package')
  await requireDirectory(baseline)
  const packageJson=async directory=>{
    const path=resolve(directory,'package.json')
    if(!(await lstat(path)).isFile())throw Error('Expected packed metadata: '+path)
    return JSON.parse(await readFile(path,'utf8'))
  }
  const [actual,expected]=await Promise.all([packageJson(unpacked),packageJson(baseline)])
  assert.equal(actual.name,expected.name)
  assert.equal(actual.version,expected.version)
  assert.deepEqual(actual.exports,expected.exports)
  async function compareDist(candidate,reference){
    await requireDirectory(reference)
    const names=(await readdir(candidate)).sort()
    assert.deepEqual(names,(await readdir(reference)).sort(),candidate)
    for(const entry of await readdir(candidate,{withFileTypes:true})){
      const relative=entry.name
      const left=resolve(candidate,relative)
      const right=resolve(reference,relative)
      if(entry.isDirectory())await compareDist(left,right)
      else if(entry.isFile()&&(await lstat(right)).isFile()){
        const [actual,expected]=await Promise.all([readFile(left,'utf8'),readFile(right,'utf8')])
        if(relative.endsWith('.json'))assert.deepEqual(JSON.parse(actual),JSON.parse(expected),relative)
        else if(actual.replaceAll('\r\n','\n')!==expected.replaceAll('\r\n','\n'))throw Error('Sealed SDK module or declaration differs: '+relative)
      }else throw Error('Unexpected packed entry: '+left)
    }
  }
  await compareDist(resolve(unpacked,'dist'),resolve(baseline,'dist'))
  console.log('SEALED_SDK_MODULES_TYPES_AND_RESOURCES_OK')
}
const destination=resolve(output,packed.filename)
await copyFile(archive,destination,constants.COPYFILE_EXCL)
console.log('PUBLIC_SDK_PACK_OK '+packed.filename+' '+packed.integrity)
console.log('PUBLIC_SDK_PACK_PATH '+destination)
