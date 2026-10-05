const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const vm=require('node:vm');
async function setup(status){
 const elements=new Map();
 const get=id=>{if(!elements.has(id))elements.set(id,{id,disabled:false,hidden:false,value:'',className:'',textContent:''});return elements.get(id);};
 const api={runtime:{onMessage:{addListener(){}},sendMessage:async msg=>msg.type==='ping'?{ok:false,error:'Specified native messaging host not found.'}:{ok:true,value:{status,message:status}}}};
 vm.runInNewContext(fs.readFileSync(require('node:path').join(__dirname,'../extension/setup.js'),'utf8'),{chrome:api,document:{getElementById:get},window:{addEventListener(){}},console});
 await get('capture').onclick();
 return get;
}
test('settings capture button remains disabled during an active capture',async()=>{const get=await setup('capturing');assert.equal(get('capture').disabled,true);assert.equal(get('cancel').hidden,false);});
test('settings capture button is enabled after completion',async()=>{const get=await setup('saved');assert.equal(get('capture').disabled,false);assert.equal(get('cancel').hidden,true);});
