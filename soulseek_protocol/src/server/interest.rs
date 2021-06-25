use crate::frame::{read_string, ParseBytes};
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendations {
    recommendations: Vec<Recommendation>,
    unrecommendations: Vec<Recommendation>,
}

impl ParseBytes for Recommendations {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let recommendation_nth = src.get_u32_le();
        let mut recommendations = vec![];

        for _ in 0..recommendation_nth {
            recommendations.push(Recommendation::parse(src)?);
        }

        let unrecommendation_nth = src.get_u32_le();
        let mut unrecommendations = vec![];

        for _ in 0..unrecommendation_nth {
            unrecommendations.push(Recommendation::parse(src)?);
        }

        Ok(Self {
            recommendations,
            unrecommendations,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemRecommendations {
    item: String,
    recommendations: Vec<Recommendation>,
}

impl ParseBytes for ItemRecommendations {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let item = read_string(src)?;

        let recommendations_nth = src.get_u32_le();
        let mut recommendations = Vec::with_capacity(recommendations_nth as usize);

        for _ in 0..recommendations_nth {
            recommendations.push(Recommendation::parse(src)?);
        }

        Ok(Self {
            item,
            recommendations,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    content: String,
    note: u32,
}

impl ParseBytes for Recommendation {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let content = read_string(src)?;
        let note = src.get_u32_le();

        Ok(Self { content, note })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interests {
    username: String,
    liked: Vec<String>,
    hated: Vec<String>,
}

impl ParseBytes for Interests {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let username = read_string(src)?;

        let liked_nth = src.get_u32_le();
        let mut liked = Vec::with_capacity(liked_nth as usize);

        for _ in 0..liked_nth {
            liked.push(read_string(src)?);
        }

        let hated_nth = src.get_u32_le();
        let mut hated = Vec::with_capacity(hated_nth as usize);

        for _ in 0..hated_nth {
            hated.push(read_string(src)?);
        }

        Ok(Self {
            username,
            liked,
            hated,
        })
    }
}
