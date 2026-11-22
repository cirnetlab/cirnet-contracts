// Inspect actual packed bytes and execute the extracted release without tools.
import assert from 'node:assert/strict'
import {createHash} from 'node:crypto'
import {constants,copyFile,lstat,mkdir,mkdtemp,readFile,readdir,writeFile} from 'node:fs/promises'
import {execFileSync} from 'node:child_process'
import {basename,dirname,isAbsolute,resolve} from 'node:path'
import {fileURLToPath,pathToFileURL} from 'node:url'
import {parseArgs} from 'node:util'
import {sourceIdentity} from './source-identity.mjs'
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..')
const {values}=parseArgs({options:{output:{type:'string'},source:{type:'string'},'node-tools':{type:'string'},baseline:{type:'string'},
  'native-bundle':{type:'string'},'native-bundle-sha256':{type:'string'},'native-crate':{type:'string'},'remote-openapi':{type:'string'}}})
if(!values.output)throw Error('An owned external --output is required')
const native=values['native-bundle'],crate=values['native-crate'],openapi=values['remote-openapi']
const manifestSha256=values['native-bundle-sha256']
const nativeMode=[native,crate,openapi,manifestSha256].some(value=>value!==undefined)
if(nativeMode&&(![native,crate,openapi].every(path=>isAbsolute(path??''))||
   !/^[a-f0-9]{64}$/.test(manifestSha256??'')||values.source||!values['node-tools'])){
  throw Error('Native SDK generation requires --native-bundle, --native-bundle-sha256, --native-crate, --remote-openapi and --node-tools')
}
const {validateArtifactDirectory}=await import(process.env.CIRNET_ARTIFACT_PATHS_MODULE??process.env.NETCIRCUIT_ARTIFACT_PATHS_MODULE??'@cirnet/artifact-paths')
const output=validateArtifactDirectory(resolve(values.output))
await mkdir(output,{recursive:true})
const run=await mkdtemp(resolve(output,'sdk-pack-'))
const packageStage=resolve(run,'sdk-source')
await mkdir(packageStage)
const digest=bytes=>createHash('sha256').update(bytes).digest('hex')
let contracts=null,remoteOpenapi=null
let sdkRoot=root,source=values.source
if(native){
  const receiptPath=resolve(native,'contracts-native-manifest.json')
  const receiptBytes=await readFile(receiptPath)
  if(digest(receiptBytes)!==manifestSha256)throw Error('Native Contracts bundle receipt SHA-256 mismatch')
  const receipt=JSON.parse(receiptBytes)
  const {revision,sourceSha256}=await sourceIdentity(root)
  const binaryPath='bin/cirnet-contracts.exe'
  const binary=resolve(native,binaryPath)
  const record=receipt.files?.find(file=>file.path===binaryPath)
  const binaryBytes=await readFile(binary)
  if(receipt.component!=='cirnet-contracts'||receipt.sourceSha256!==sourceSha256||!record||
     record.size!==binaryBytes.length||record.sha256!==digest(binaryBytes))throw Error('Native Contracts bundle differs from its source or binary receipt')
  const archive=await readFile(crate)
  const crateName=`cirnet-contracts-native-${receipt.version}`
  if(!/^cirnet-contracts-native-\d+\.\d+\.\d+-dev\.\d+$/.test(crateName)||basename(crate)!==crateName+'.crate'||
     receipt.crate?.filename!==basename(crate)||receipt.crate.size!==archive.length||receipt.crate.sha256!==digest(archive)){
    throw Error('Native Contracts crate differs from the bundle receipt')
  }
  const tar=(...args)=>execFileSync('tar',args,{input:archive,windowsHide:true})
  const members=tar('-tzf','-').toString('utf8').trim().split(/\r?\n/)
  if(members.some(name=>!name.startsWith(crateName+'/')||name.split('/').some(part=>part==='..'||/[\\:]/.test(part)))||
     tar('-tvzf','-').toString('utf8').trim().split(/\r?\n/).some(line=>!/^[-d]/.test(line))){
    throw Error('Unexpected member in the selected Contracts crate')
  }
  tar('-xzf','-','-C',run)
  sdkRoot=resolve(run,crateName)
  for(const file of receipt.files){
    if(!file.path.startsWith('resources/'))continue
    const path=file.path.slice('resources/'.length)
    if(!/^(contracts|device-library)\/(?:[a-zA-Z0-9_.-]+\/)*[a-zA-Z0-9_.-]+\.(json|proto)$/.test(path)||
       path.split('/').some(part=>part==='.'||part==='..'))throw Error('Unexpected Contracts resource path: '+path)
    const bytes=await readFile(resolve(sdkRoot,path))
    if(bytes.length!==file.size||digest(bytes)!==file.sha256)throw Error('Contracts crate disagrees with the runtime resource receipt: '+path)
  }
  const handoff=await readFile(openapi)
  const lockedOpenapi=resolve(run,'remote-openapi.json')
  await writeFile(lockedOpenapi,handoff,{flag:'wx'})
  const projection=resolve(run,'native-projection')
  execFileSync(binary,['generate-sdk',sdkRoot,projection,resolve(values['node-tools']),lockedOpenapi],{cwd:run,stdio:'pipe',windowsHide:true})
  source=resolve(projection,'contracts/browser')
  contracts={revision,sourceSha256,manifestSha256,crateSha256:receipt.crate.sha256}
  remoteOpenapi={sha256:digest(handoff)}
}
for(const name of ['package.json','README.md','LICENSE'])await copyFile(resolve(sdkRoot,'packages/public-sdk',name),resolve(packageStage,name))
const buildArgs=[resolve(sdkRoot,'packages/public-sdk/scripts/build.mjs'),'--output',resolve(packageStage,'dist')]
if(source)buildArgs.push('--source',resolve(source))
if(values['node-tools'])buildArgs.push('--node-tools',resolve(values['node-tools']))
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
if(native){
  const bytes=await readFile(destination)
  const sdk={filename:packed.filename,version:packed.version,size:bytes.length,sha256:digest(bytes),integrity:packed.integrity}
  await writeFile(resolve(output,'native-sdk-receipt.json'),JSON.stringify({schemaVersion:1,contracts,sdk,remoteOpenapi},null,2)+'\n',{flag:'wx'})
}
console.log('PUBLIC_SDK_PACK_OK '+packed.filename+' '+packed.integrity)
console.log('PUBLIC_SDK_PACK_PATH '+destination)
