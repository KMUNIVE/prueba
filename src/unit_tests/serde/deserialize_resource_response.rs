use crate::types::addon::ResourceResponse::{Meta, Metas, MetasDetailed};
use crate::types::addon::{DescriptorPreview, ManifestPreview};
use crate::types::resource::{
    Link, MetaItem, MetaItemBehaviorHints, MetaItemPreview, SeriesInfo, Subtitles, Video,
};
use chrono::prelude::TimeZone;
use chrono::Utc;
use semver::Version;
use url::Url;

#[test]
fn deserialize_resource_response_metas() {
    let metas_vec = vec![MetaItemPreview {
        id: "id".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: None,
        logo: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        trailer_streams: vec![],
        behavior_hints: MetaItemBehaviorHints {
            default_video_id: None,
            featured_video_id: None,
            has_scheduled_videos: false,
        },
    }];
    let metas_json = r#"
    {
        "metas": [
            {
                "id": "id",
                "type": "type_name",
                "name": "name",
                "poster": null,
                "logo": null,
                "description": null,
                "releaseInfo": null,
                "runtime": null,
                "released": null,
                "posterShape": "poster",
                "trailers": [],
                "behaviorHints": {
                    "defaultVideoId": null,
                    "featuredVideoId": null
                }
            }
        ]
    }
    "#;
    let metas_deserialize = serde_json::from_str(&metas_json).unwrap();
    match metas_deserialize {
        Metas { metas } => assert_eq!(metas, metas_vec, "metas deserialized successfully"),
        _ => panic!("failed getting metas"),
    };
}

#[test]
fn deserialize_resource_response_metas_detailed() {
    let metas_detailed_vec = vec![MetaItem {
        id: "id".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: None,
        background: None,
        logo: None,
        popularity: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        videos: vec![Video {
            id: "id".to_owned(),
            title: "title".to_owned(),
            released: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            overview: None,
            thumbnail: None,
            streams: vec![],
            series_info: Some(SeriesInfo {
                season: 1,
                episode: 1,
            }),
            trailer_streams: vec![],
        }],
        links: vec![Link {
            name: "name".to_owned(),
            category: "category".to_owned(),
            url: Url::parse("https://url").unwrap(),
        }],
        trailer_streams: vec![],
        behavior_hints: Default::default(),
    }];
    let metas_detailed_json = r#"
    {
        "metasDetailed": [
            {
                "id": "id",
                "type": "type_name",
                "name": "name",
                "poster": null,
                "background": null,
                "logo": null,
                "popularity": null,
                "description": null,
                "releaseInfo": null,
                "runtime": null,
                "released": null,
                "posterShape": "poster",
                "videos": [
                    {
                        "id": "id",
                        "title": "title",
                        "released": "2020-01-01T00:00:00Z",
                        "overview": null,
                        "thumbnail": null,
                        "streams": [],
                        "season": 1,
                        "episode": 1,
                        "trailers": []
                    }
                ],
                "links": [
                    {
                        "name": "name",
                        "category": "category",
                        "url": "https://url/"
                    }
                ],
                "trailers": [],
                "behaviorHints": {
                    "defaultVideoId": null,
                    "featuredVideoId": null
                }
            }
        ]
    }
    "#;
    let metas_detailed_deserialize = serde_json::from_str(&metas_detailed_json).unwrap();
    match metas_detailed_deserialize {
        MetasDetailed { metas_detailed } => assert_eq!(
            metas_detailed, metas_detailed_vec,
            "metas detailed deserialized successfully"
        ),
        _ => panic!("failed getting metas detailed"),
    };
}

#[test]
fn deserialize_resource_response_meta() {
    let meta_item = MetaItem {
        id: "id".to_owned(),
        type_name: "type_name".to_owned(),
        name: "name".to_owned(),
        poster: None,
        background: None,
        logo: None,
        popularity: None,
        description: None,
        release_info: None,
        runtime: None,
        released: None,
        poster_shape: Default::default(),
        videos: vec![Video {
            id: "id".to_owned(),
            title: "title".to_owned(),
            released: Some(Utc.ymd(2020, 1, 1).and_hms_milli(0, 0, 0, 0)),
            overview: None,
            thumbnail: None,
            streams: vec![],
            series_info: Some(SeriesInfo {
                season: 1,
                episode: 1,
            }),
            trailer_streams: vec![],
        }],
        links: vec![Link {
            name: "name".to_owned(),
            category: "category".to_owned(),
            url: Url::parse("https://url").unwrap(),
        }],
        trailer_streams: vec![],
        behavior_hints: Default::default(),
    };
    let meta_json = r#"
    {
        "meta": {
            "id": "id",
            "type": "type_name",
            "name": "name",
            "poster": null,
            "background": null,
            "logo": null,
            "popularity": null,
            "description": null,
            "releaseInfo": null,
            "runtime": null,
            "released": null,
            "posterShape": "poster",
            "videos": [
                {
                    "id": "id",
                    "title": "title",
                    "released": "2020-01-01T00:00:00Z",
                    "overview": null,
                    "thumbnail": null,
                    "streams": [],
                    "season": 1,
                    "episode": 1,
                    "trailers": []
                }
            ],
            "links": [
                {
                    "name": "name",
                    "category": "category",
                    "url": "https://url/"
                }
            ],
            "trailers": [],
            "behaviorHints": {
                "defaultVideoId": null,
                "featuredVideoId": null
            }
        }
    }
    "#;
    let meta_deserialize = serde_json::from_str(&meta_json).unwrap();
    match meta_deserialize {
        Meta { meta } => assert_eq!(meta, meta_item, "meta deserialized successfully"),
        _ => panic!("failed getting meta"),
    };
}

#[test]
fn deserialize_resource_response_subtitles() {
    let subtitles = vec![Subtitles {
        id: "id".to_owned(),
        lang: "lang".to_owned(),
        url: Url::parse("https://url").unwrap(),
    }];
    let subtitles_json = r#"
    [
        {
            "id": "id",
            "lang": "lang",
            "url": "https://url/"
        }
    ]
    "#;
    let subtitles_deserialize: Vec<Subtitles> = serde_json::from_str(&subtitles_json).unwrap();
    assert_eq!(
        subtitles, subtitles_deserialize,
        "subtitles deserialized successfully"
    );
}

#[test]
fn deserialize_resource_response_addons() {
    let descriptor_previews = vec![
        DescriptorPreview {
            manifest: ManifestPreview {
                id: "id1".to_owned(),
                version: Version::new(0, 0, 1),
                name: "name".to_owned(),
                description: Some("description".to_owned()),
                logo: Some("logo".to_owned()),
                background: Some("background".to_owned()),
                types: vec!["type".to_owned()],
            },
            transport_url: Url::parse("https://transport_url").unwrap(),
        },
        DescriptorPreview {
            manifest: ManifestPreview {
                id: "id2".to_owned(),
                version: Version::new(0, 0, 1),
                name: "name".to_owned(),
                description: None,
                logo: None,
                background: None,
                types: vec![],
            },
            transport_url: Url::parse("https://transport_url").unwrap(),
        },
    ];
    let descriptor_previews_json = r#"
    [
        {
            "manifest": {
                "id": "id1",
                "version": "0.0.1",
                "name": "name",
                "description": "description",
                "logo": "logo",
                "background": "background",
                "types": [
                    "type"
                ]
            },
            "transportUrl": "https://transport_url/"
        },
        {
            "manifest": {
                "id": "id2",
                "version": "0.0.1",
                "name": "name",
                "description": null,
                "logo": null,
                "background": null,
                "types": []
            },
            "transportUrl": "https://transport_url/"
        }
    ]
    "#;
    let descriptor_previews_deserialize: Vec<DescriptorPreview> =
        serde_json::from_str(&descriptor_previews_json).unwrap();
    assert_eq!(
        descriptor_previews, descriptor_previews_deserialize,
        "descriptor previews deserialized successfully"
    );
}