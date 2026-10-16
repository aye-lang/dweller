var tall = document.createElement('div');
tall.className = 'tall';
tall.textContent = 'A tall part.';
document.body.appendChild(tall);
document.getElementById('log').textContent = 'ran ' + location.pathname;
