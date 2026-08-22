(function () {
    function formatLabel(value) {
        if (!value) return '';
        return value.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
    }

    function closeAll(except) {
        document.querySelectorAll('[data-custom-select-menu]').forEach((menu) => {
            if (!except || menu !== except) {
                menu.classList.add('hidden');
            }
        });
        document.querySelectorAll('[data-custom-select-trigger]').forEach((trigger) => {
            trigger.setAttribute('aria-expanded', 'false');
        });
    }

    function initCustomSelect(root) {
        const hidden = root.querySelector('input[type="hidden"]');
        const trigger = root.querySelector('[data-custom-select-trigger]');
        const menu = root.querySelector('[data-custom-select-menu]');
        const label = root.querySelector('[data-custom-select-label]');
        const options = root.querySelectorAll('[data-custom-select-option]');

        if (!hidden || !trigger || !menu || !label) return;

        function highlightSelected(value) {
            options.forEach((opt) => {
                const selected = opt.dataset.value === value;
                opt.classList.toggle('bg-brand-50', selected);
                opt.classList.toggle('text-brand-700', selected);
                opt.classList.toggle('font-medium', selected);
            });
        }

        function setValue(value, text) {
            hidden.value = value;
            label.textContent = text || formatLabel(value) || 'Select…';
            highlightSelected(value);
            hidden.dispatchEvent(new Event('change', { bubbles: true }));
        }

        const initial = Array.from(options).find((o) => o.dataset.value === hidden.value);
        if (initial) {
            setValue(initial.dataset.value, initial.dataset.label || initial.textContent.trim());
        } else if (hidden.value) {
            label.textContent = formatLabel(hidden.value);
            highlightSelected(hidden.value);
        } else {
            label.textContent = label.dataset.placeholder || 'Select…';
            highlightSelected('');
        }

        trigger.addEventListener('click', (e) => {
            e.preventDefault();
            e.stopPropagation();
            const isOpen = !menu.classList.contains('hidden');
            closeAll();
            if (!isOpen) {
                menu.classList.remove('hidden');
                trigger.setAttribute('aria-expanded', 'true');
            }
        });

        options.forEach((opt) => {
            opt.addEventListener('click', (e) => {
                e.preventDefault();
                setValue(opt.dataset.value, opt.dataset.label || opt.textContent.trim());
                menu.classList.add('hidden');
                trigger.setAttribute('aria-expanded', 'false');
            });
        });
    }

    document.addEventListener('click', () => closeAll());

    document.addEventListener('DOMContentLoaded', () => {
        document.querySelectorAll('[data-custom-select]').forEach(initCustomSelect);
    });
})();
