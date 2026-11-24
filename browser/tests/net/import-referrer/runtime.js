import { name } from './static.js';

log('static import: ' + name);
import('./late.js').then(late => log('dynamic import: ' + late.name), e => log('dynamic import: ' + e.name)).then(next);
