CREATE TABLE IF NOT EXISTS shops (
    id UUID PRIMARY KEY,
    slug VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    shop_type VARCHAR(30) NOT NULL,
    is_system BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO shops (id, slug, name, shop_type, is_system) VALUES
    ('b0000001-0001-4000-8000-000000000001', 'aeon', 'Aeon', 'supermarket', TRUE),
    ('b0000001-0001-4000-8000-000000000002', 'seiyu', 'Seiyu', 'supermarket', TRUE),
    ('b0000001-0001-4000-8000-000000000003', 'life', 'Life', 'supermarket', TRUE),
    ('b0000001-0001-4000-8000-000000000004', 'ok_store', 'OK Store', 'supermarket', TRUE),
    ('b0000001-0001-4000-8000-000000000005', 'gyomu_super', 'Gyomu Super', 'wholesale', TRUE),
    ('b0000001-0001-4000-8000-000000000006', 'seijo_ishii', 'Seijo Ishii', 'specialty', TRUE),
    ('b0000001-0001-4000-8000-000000000007', 'costco', 'Costco Japan', 'wholesale', TRUE),
    ('b0000001-0001-4000-8000-000000000008', 'ito_yokado', 'Ito Yokado', 'supermarket', TRUE),
    ('b0000001-0001-4000-8000-000000000009', 'my_basket', 'My Basket', 'supermarket', TRUE),
    ('b0000001-0001-4000-8000-000000000010', 'maruetsu', 'Maruetsu', 'supermarket', TRUE),
    ('b0000001-0001-4000-8000-000000000011', 'kaldi', 'Kaldi Coffee Farm', 'specialty', TRUE),
    ('b0000001-0001-4000-8000-000000000012', 'national_azabu', 'National Azabu', 'specialty', TRUE),
    ('b0000001-0001-4000-8000-000000000013', 'seven_eleven', '7-Eleven', 'convenience', TRUE),
    ('b0000001-0001-4000-8000-000000000014', 'lawson', 'Lawson', 'convenience', TRUE),
    ('b0000001-0001-4000-8000-000000000015', 'familymart', 'FamilyMart', 'convenience', TRUE),
    ('b0000001-0001-4000-8000-000000000016', 'don_quijote', 'Don Quijote', 'supermarket', TRUE),
    ('b0000001-0001-4000-8000-000000000017', 'uniqlo', 'Uniqlo', 'specialty', TRUE),
    ('b0000001-0001-4000-8000-000000000018', 'muji', 'Muji', 'specialty', TRUE),
    ('b0000001-0001-4000-8000-000000000019', 'nitori', 'Nitori', 'specialty', TRUE),
    ('b0000001-0001-4000-8000-000000000020', 'ikea', 'IKEA', 'specialty', TRUE),
    ('b0000001-0001-4000-8000-000000000021', 'matsumoto_kiyoshi', 'Matsumoto Kiyoshi', 'specialty', TRUE),
    ('b0000001-0001-4000-8000-000000000022', 'daikoku_drug', 'Daikoku Drug', 'specialty', TRUE),
    ('b0000001-0001-4000-8000-000000000023', 'other', 'Other Shop', 'supermarket', TRUE);
