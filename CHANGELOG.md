# Changelog

All notable changes to this project will be documented in this file.

## [0.3.7] - 2025-03-20

### 🐛 Bug Fixes

- Ana projeden deadpool-postrges cratesi export edildi.

### 🚜 Refactor

- Bu commit, parsql-sqlite paketinde metod isimlerini diğer parsql paketleriyle tutarlı hale getirmek amacıyla aşağıdaki değişiklikleri içermektedir:Yapılan Değişiklikler

* Kod Değişiklikleri

- CrudOps trait'inde get ve get_all metodları fetch ve fetch_all olarak yeniden adlandırıldı
- Eski metodlar geriye dönük uyumluluk için #[deprecated] olarak işaretlendi
- transactional_ops.rs dosyasında tx_get ve tx_get_all fonksiyonları tx_fetch ve tx_fetch_all olarak yeniden adlandırıldı
- lib.rs dosyasında yeni metod adları dışa aktarıldı

* Belgelendirme Değişiklikleri

- README.md ve README.en.md dosyalarındaki tüm örnekler ve açıklamalar güncellendi
- Metod isimlerinin değiştirilmesiyle ilgili referanslar güncellendi
- Örneklerde get yerine fetch kullanımı gösterildi

* Diğer İyileştirmeler

- Metod belgelerinde (doc comments) güncelleme ve iyileştirmeler yapıldı
- Belgelendirmelerde bölüm isimleri ve başlıklar tutarlı hale getirildi
- Türkçe ve İngilizce belgeler senkronize edildi

Bu değişiklikler, parsql ekosistemindeki tüm paketlerin tutarlı bir isimlendirme düzenine sahip olmasını sağlamaktadır. Geriye dönük uyumluluk korunduğu için mevcut kodların çalışmaya devam etmesi garanti edilmiştir.

### 📚 Documentation

- Versiyon ile ilgili dökümantasyon güncellemesi.
- V0.3.7 için CHANGELOG.md düzenlemesi.

### ⚙️ Miscellaneous Tasks

- *(release)* Bugfix için versiyon yükseltme.
- *(release)* Deadpool-postgres için export durumundan kaynaklı versiyon yükseltme.
- *(release)* Versiyon problemleri...

## [0.3.4] - 2025-03-14

### 🐛 Bug Fixes

- Parsql-tokio-postgres ve parsql-deadpool-postgres crate'lerindeki extension metodların aseknron desteği güncellendi.

### ⚙️ Miscellaneous Tasks

- *(release)* V0.3.3 için hazırlıklar.

## [0.3.3] - 2025-03-14

### 🚀 Features

- Artık  işlemleri, ilgili cratelerin veritabanı istemci nesneleri üzerinde bir  metod gibi çağırılabilecek.
- Bağlantı havuzu yapısına daha etkin destek için,  cratesi oluşturuldu. Transactional işlemler içinde daha efektif destekler eklendi.
- Derive makrosuna sayfalama desteği eklemek için  ve  öznitelikleri eklendi.
- Artık  işlemleri, ilgili cratelerin veritabanı istemci nesneleri üzerinde bir extension metod gibi çağırılabilecek. Ek olarak crate'lere transactional işlem desteği eklendi.
- Bağlantı havuzu yapısına daha etkin destek için 'parsql-deadpool-posgres' cratesi oluşturuldu. crud işlemleri, 'Pool' struct'ı için extension olarak genişletildi. Transactional işlemler içinde daha efektif destekler eklendi.
- 'Queryable' derive makrosuna sayfalama desteği eklemek için 'limit' ve 'offset' öznitelikleri eklendi.

### 🚜 Refactor

- 'parsql-tokio-postgres' paketi içinde bir özellik olarak ele aldığımız 'deadpool-postgres' için ayrı bir paket üzerinden daha detaylı tüm özellikleri sağlayacak şekilde yeniden düzenleme çalışması yapıldı.

### ⚙️ Miscellaneous Tasks

- *(release)* Paket dökümantasyon düzenlemeleri.

## [0.3.2] - 2025-03-09

### 🚀 Features

- *(core)* Basit crud operasyonları için struct üzerinde tanımlanacak ilgili 'procedural' makrolar ve 'attribute'ler ile işlem desteği
- SqlParams trait implement eden macro ayrıştırıldı, 'FromRow' traiti ve macrosu eklendi
- *(macros)* FromRow trait'ini üreten 'derive' makrosuna son hali verildi, 'parsql-macros' cratesi feature bazlı yapıya göre revize edildi
- 'join' özniteliği ile daha karmaşık sorgular mümkün hale getirilmiştir.
- Environment variable 'PARSQL_TRACE=1' olarak işaretli haldeyken çalıştırılırsa, trace loglarında oluşturulan sql sorgularının console'da gösterilmesini sağlayacak özellikler paketlere eklendi.
- Daha gelişmiş sorgulara imkan tanımak için 'group_by', 'order_by' ve 'having' özniteliği 'Queryable' derive makrosuna eklendi.

### 🚜 Refactor

- *(macros)* Makrolarda, compiler tarafında bir uyarı verilmesine sebep olan durum giderilip, 'ToSql' için çözüm netleştirildi.
- *(macros)* Makroların 'features'leri ile ilgili düzenlemeler yapıldı.
- 'parsql-core' küfesi oluşturularak trait'ler buraya taşındı, 'parsql-tokio-postgres' küfesi oluşturularak ilgili özellikler bu küfeye taşındı, ilgili küfelere benchmark'lar hazırlandı.
- Parsql-core'da bulunan crud işlemleri için hazırlanan trait'ler kaldırıldı. Bunun yerine ilgili crate'lere 'SqlQuery' trait'i eklenerek daha sade bir yapı için adım atıldı.
- 'SqlQuery', 'SqlParams', 'UpdateParams' ve 'FromRow' traitleri 'parsql-core' cratesine taşındı, feature tipine göre gerekli düzenlemeler yapıldı.
- Makroların öz niteliklerinin isim değişikliklerini barındırır. table_name=table, update_clause=update, select_clause=select. Ek olarak ortak trait'lerini core paketine taşınmıştır.
- Yapıda genel düzenleme çalışmaları yapıldı.
- FromRow makrosu üzerinde bazı düzenleme ve iyileştirmeler yapıldı.
- 'Deleteable' ifadesi 'Deletable' olarak değiştirildi ve dökümantasyon ingilizceye çevirildi.

### 📚 Documentation

- CHANGELOG.md güncellendi
- CHANGELOG.md güncellendi.
- Küfelerin dökümantasyonları için README.md dosyaları eklendi ve içerik eklendi.
- Ana küfe için Cargo.toml dosyasına açıklama eklendi.
- README.md dökümantasyonları yeni versiyona göre güncellendi.
- CHANGELOG.md düzenlendi
- Crate'lerdeki metodlara açıklayıcı yorumlar eklenmiştir.
- CHANGELOG.md güncellendi.
- Genel olarak proje belgeleri yeni versiyona göre güncellendi ve re-organize edildi.
- Paketlerdeki metodların yorumları yeni geliştirmelere göre güncellendi.
- README.md de düzenleme yapıldı.
- Dökümantasyonlarda güncellemeler yapıldı, CHANGELOG.md güncellendi.
- Dökümantasyon güncellemeleri ve versiyon yükseltme v0.3.2
- Dökümantasyon güncellemeleri için düzenlemeler.

### ⚙️ Miscellaneous Tasks

- Paketlerin cargo toml dosyalarında düzenlemeler yapıldı.
- *(release)* Yayın öncesi paket versiyon hazırlıkları.
- *(release)* Yayınlanmış olan 0.3.1 versiyonu için örnek projelerde düzenlemeler yapıldı.

<!-- generated by git-cliff -->
