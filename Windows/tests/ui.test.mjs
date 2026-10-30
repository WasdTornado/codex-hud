import test from 'node:test';
import assert from 'node:assert/strict';
import {digits,halfPath,Gesture,name} from '../ui/geometry.js';
test('未知、零值、舍入与三位数',()=>{assert.equal(digits(null),'—');assert.equal(digits(0),'0');assert.equal(digits(99.8),'100');assert.equal(digits(NaN),'—');assert.equal(name(60),'60 分钟');});
test('胶囊进度只走外缘，不穿越中央',()=>{for(const lower of [false,true]){for(const fraction of [0,.1,.5,1]){const points=halfPath(fraction,lower).match(/[ML]([^ML]+)/g).map(p=>p.slice(1).split(',').map(Number));for(const [x,y]of points){assert(x>=2.49&&x<=33.51);assert(lower?y>=37.99:y<=34.01);}}}});
test('点击与拖动互斥，丢失捕获不触发点击',()=>{const g=new Gesture();g.down(10,10);assert(!g.move(12,12));assert(g.up());g.down(10,10);assert(g.move(14,10));assert(!g.move(15,12));assert(!g.up());g.down(0,0);g.cancel();assert(!g.up());});
